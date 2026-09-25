# postgresql-setup.md — ติดตั้ง PostgreSQL สำหรับ OSCC บน Windows

> เอกสารนี้บันทึก **คำสั่งจริงทั้งหมด** ที่ใช้ติดตั้ง PostgreSQL 18.6 บนเครื่องพัฒนา
> (Windows, locale ไทย) พร้อมเหตุผลของแต่ละขั้น เพื่อให้ทำซ้ำหรือย้ายไปเครื่องอื่นได้
> รหัสผ่านในเอกสารนี้เป็น **ตัวอย่าง** — ของจริงให้เก็บใน password manager เท่านั้น

## ภาพรวมสิ่งที่ติดตั้ง

| องค์ประกอบ | ค่าที่ใช้ |
|---|---|
| เวอร์ชัน | PostgreSQL 18.6 (EDB installer `postgresql-18.6-4-windows-x64.exe`) |
| ที่ติดตั้ง | `C:\Program Files\PostgreSQL\18` |
| data directory | `C:\Program Files\PostgreSQL\18\data` |
| Service | `postgresql-x64-18` (Running / Automatic, รันด้วย `NT AUTHORITY\NetworkService`) |
| Port | `5432` (listen เฉพาะ localhost) |
| Cluster encoding | `UTF8` (ตรวจสอบแล้ว) |
| Superuser | `postgres` |
| App role | `oscc_app` (LOGIN, ไม่ใช่ superuser) |
| Databases | `oscc` (ใช้งานจริง) และ `oscc_test` (สำหรับ DB-backed tests) |

หลักคิด: **แยก role ของแอปออกจาก superuser** — แอปเชื่อมด้วย `oscc_app` เท่านั้น
ส่วน `postgres` ใช้เฉพาะงานติดตั้ง/ดูแล

---

## ขั้นที่ 1 — ดูข้อมูล package และ URL ทางการ

```powershell
winget show --id PostgreSQL.PostgreSQL.18 --exact
```

ผลลัพธ์สำคัญ: `Installer Url` ชี้ไปที่ EDB และมี `Installer SHA256` ให้ตรวจ
(winget manifest ใช้ silent switches `--mode unattended --unattendedmodeui none`
โดยไม่ตั้งรหัส superuser — เราจึงติดตั้งเองเพื่อกำหนดรหัสและ path ได้)

## ขั้นที่ 2 — ดาวน์โหลดและตรวจ SHA256

```powershell
$url = "https://get.enterprisedb.com/postgresql/postgresql-18.6-4-windows-x64.exe"
$installer = "$env:TEMP\postgresql-18.6-4-windows-x64.exe"
Invoke-WebRequest -Uri $url -OutFile $installer -UseBasicParsing

(Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash.ToLower()
# ต้องได้: a98074954015f3d733dbb92315bd06e01a381778f0a8ae4cebf9cb199d73aab0
```

**ทำไมต้องตรวจ**: ไฟล์ installer คือโค้ดที่จะรันด้วยสิทธิ์ admin — ตรวจ hash กับ
manifest ทางการก่อนรันเสมอ

## ขั้นที่ 3 — ติดตั้งแบบ unattended

```powershell
$args = @(
  '--mode','unattended',
  '--unattendedmodeui','none',
  '--superpassword','<SUPERUSER_PASSWORD>',
  '--serverport','5432',
  '--servicename','postgresql-x64-18',
  '--prefix','"C:\Program Files\PostgreSQL\18"',
  '--datadir','"C:\Program Files\PostgreSQL\18\data"',
  '--enable-components','server,commandlinetools'
)
$proc = Start-Process -FilePath $installer -ArgumentList $args -Verb RunAs -Wait -PassThru
$proc.ExitCode   # ต้องได้ 0
```

คำอธิบาย flag:

| Flag | ความหมาย |
|---|---|
| `--mode unattended` | ติดตั้งแบบไม่ถามอะไร |
| `--unattendedmodeui none` | ไม่ต้องมีหน้าต่างความคืบหน้า |
| `--superpassword` | รหัสของ user `postgres` |
| `--serverport` | port ของ service |
| `--servicename` | ชื่อ Windows service |
| `--prefix` | โฟลเดอร์ติดตั้ง |
| `--datadir` | โฟลเดอร์เก็บข้อมูล (คนละที่กับตัวโปรแกรม) |
| `--enable-components` | ติดตั้งเฉพาะ server + command-line tools |

`-Verb RunAs` ทำให้ **UAC เด้งขึ้นให้กด Yes** เพราะการติดตั้ง service ต้องใช้สิทธิ์ admin

### บทเรียนจากการรันจริง (รอบแรก fail)

รอบแรกได้ `exit code: 1` ภายใน 12 วินาที สาเหตุคือ
**PowerShell 5.1 ไม่ใส่ quote ให้ argument ที่มีช่องว่างในตัว** เมื่อใช้
`Start-Process -ArgumentList` เป็น array — command line ที่ส่งจริงกลายเป็น
`--prefix C:\Program Files\PostgreSQL\18` แล้ว installer อ่านเป็นหลาย argument

แก้โดย **ใส่ quote ไว้ใน string เอง**: `'"C:\Program Files\PostgreSQL\18"'`

ถ้าติดตั้งล้มอีก ให้เพิ่ม `--debuglevel 4 --debugtrace "$env:TEMP\pg-install-trace.log"`
เพื่อดู trace (แต่รอบนี้ trace ไม่ถูกสร้าง เพราะล้มตั้งแต่ parse argument)

**ผลลัพธ์จริง**: exit code `0`, ใช้เวลา 180 วินาที

## ขั้นที่ 4 — ตรวจสอบหลังติดตั้ง

```powershell
Test-Path "C:\Program Files\PostgreSQL\18\bin\psql.exe"   # True
Get-Service postgresql-x64-18 | Format-List Name, Status, StartType
& "C:\Program Files\PostgreSQL\18\bin\psql.exe" --version  # psql (PostgreSQL) 18.6
Get-Content "C:\Program Files\PostgreSQL\18\installation_summary.log" -Tail 15
```

ตรวจ encoding ของ cluster (สำคัญ: locale ไทยอาจทำให้ได้ encoding เก่า):

```powershell
$env:PGPASSWORD='<SUPERUSER_PASSWORD>'
& psql -U postgres -h 127.0.0.1 -p 5432 -c "SELECT datname, pg_encoding_to_char(encoding) AS encoding, datcollate FROM pg_database;"
Remove-Item Env:PGPASSWORD
```

เครื่องนี้ได้ `UTF8` ทุก database — PostgreSQL 18 จัดการให้อัตโนมัติ
ถ้าเครื่องอื่นได้ `WIN874`/`WIN1252` ให้สร้าง DB ของแอปด้วย
`TEMPLATE template0 ENCODING 'UTF8'` แทน

> หมายเหตุ PG 18: คำสั่ง `SHOW lc_collate;` ถูกถอดแล้ว — ให้อ่านจาก `pg_database` แทน

## ขั้นที่ 5 — เพิ่ม `psql` เข้า PATH (ระดับ user)

```powershell
$bin = 'C:\Program Files\PostgreSQL\18\bin'
$userPath = [Environment]::GetEnvironmentVariable('Path','User')
if ($userPath -notlike "*PostgreSQL\18\bin*") {
  [Environment]::SetEnvironmentVariable('Path', "$userPath;$bin", 'User')
}
```

เปิด terminal ใหม่แล้วจะใช้ `psql` ได้เลย (ไม่ต้องใส่ full path)

## ขั้นที่ 6 — สร้าง role และ database แบบ least privilege

```powershell
$env:PGPASSWORD='<SUPERUSER_PASSWORD>'
& psql -U postgres -h 127.0.0.1 -p 5432 -v ON_ERROR_STOP=1 -c "CREATE ROLE oscc_app LOGIN PASSWORD '<APP_PASSWORD>';"
& psql -U postgres -h 127.0.0.1 -p 5432 -v ON_ERROR_STOP=1 -c "CREATE DATABASE oscc OWNER oscc_app;"
& psql -U postgres -h 127.0.0.1 -p 5432 -v ON_ERROR_STOP=1 -c "CREATE DATABASE oscc_test OWNER oscc_app;"
Remove-Item Env:PGPASSWORD
```

ตรวจว่า role ใช้งานได้จริง (**ระวัง**: `PGPASSWORD` ต้องเป็นรหัสของ role นั้น
ไม่ใช่ของ superuser):

```powershell
$env:PGPASSWORD='<APP_PASSWORD>'
& psql -U oscc_app -h 127.0.0.1 -p 5432 -d oscc -c "SELECT current_user, current_database();"
Remove-Item Env:PGPASSWORD
```

**ทำไม `oscc_app` เป็นเจ้าของ DB**: migration สร้างตารางและ index ต้องใช้สิทธิ์ owner
ส่วน production จริงควรแยก `oscc_migrator` (owner, ใช้ตอน deploy) ออกจาก
`oscc_app` (INSERT/SELECT/DELETE เฉพาะตารางที่ต้องใช้) — ดู `docs/runbook.md`

## ขั้นที่ 7 — ให้แอปรัน migration + สร้างผู้ใช้คนแรก

```powershell
# 1) build ก่อน เพื่อไม่ให้ cargo กลายเป็นพ่อของ process
cargo build -p oscc-server --example hash_password

# 2) สตาร์ท API — migration จะรันอัตโนมัติตอนเปิด
$env:OSCC_DATABASE_URL='postgres://oscc_app:<APP_PASSWORD>@127.0.0.1:5432/oscc'
$proc = Start-Process -FilePath "target\debug\oscc-server.exe" `
  -WorkingDirectory (Get-Location) -WindowStyle Hidden `
  -RedirectStandardOutput "$env:TEMP\oscc-server.out.log" `
  -RedirectStandardError  "$env:TEMP\oscc-server.err.log" -PassThru
"server PID: $($proc.Id)"

# 3) ตรวจสุขภาพ — ต้องได้ database: true
Invoke-RestMethod -Uri "http://127.0.0.1:8080/healthz"
```

สร้าง hash รหัสผ่าน (อ่านจาก **stdin** เพื่อไม่ให้ติด shell history):

```powershell
$hash = ('<USER_PASSWORD>' | & "target\debug\examples\hash_password.exe").Trim()
```

ใส่ผู้ใช้คนแรก:

```powershell
$env:PGPASSWORD='<APP_PASSWORD>'
$sql = "INSERT INTO users (username, display_name, role, password_hash) VALUES ('nurse.a', 'Nurse A', 'er_nurse', '$hash') ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash;"
& psql -U oscc_app -h 127.0.0.1 -p 5432 -d oscc -c $sql
Remove-Item Env:PGPASSWORD
```

role ที่ใช้ได้: `er_nurse`, `forensic_physician`, `social_worker`,
`psychologist`, `oscc_lead`, `admin` (ดู `docs/rbac.md`)

## ขั้นที่ 8 — ทดสอบ login จริงผ่าน HTTP

```powershell
$body = @{ username='nurse.a'; password='<USER_PASSWORD>' } | ConvertTo-Json
$login = Invoke-RestMethod -Uri "http://127.0.0.1:8080/api/v1/auth/login" `
  -Method Post -ContentType 'application/json' -Body $body
$login.user

$me = Invoke-RestMethod -Uri "http://127.0.0.1:8080/api/v1/auth/me" `
  -Headers @{ Authorization = "Bearer $($login.token)" }
$me

$env:PGPASSWORD='<APP_PASSWORD>'
& psql -U oscc_app -h 127.0.0.1 -p 5432 -d oscc -c "SELECT id, action, actor, actor_role FROM audit_entries ORDER BY id;"
Remove-Item Env:PGPASSWORD
```

ต้องเห็น audit entry `login_succeeded` — ถ้าไม่มี แปลว่ามีอะไรผิดในเส้นทาง audit

## ขั้นที่ 9 — รัน DB-backed tests

```powershell
$env:OSCC_TEST_DATABASE_URL='postgres://oscc_app:<APP_PASSWORD>@127.0.0.1:5432/oscc_test'
cargo test -p oscc-server --test auth_db
Remove-Item Env:OSCC_TEST_DATABASE_URL
```

เทสต์นี้รัน flow จริงทั้งชุด (login → me → logout → ถูกปฏิเสธ) แล้วลบข้อมูลทดสอบเอง
ถ้าไม่ตั้ง env ตัวนี้ เทสต์จะ **ข้าม** เพื่อให้ชุดทดสอบปกติไม่ต้องพึ่ง DB

---

## งานประจำวัน

```powershell
# สถานะ / เริ่ม / หยุด service (start/stop ต้องรัน PowerShell as Administrator)
Get-Service postgresql-x64-18
Start-Service postgresql-x64-18
Stop-Service postgresql-x64-18

# เปิด psql เข้า DB ของแอป
$env:PGPASSWORD='<APP_PASSWORD>'
psql -U oscc_app -h 127.0.0.1 -p 5432 -d oscc

# หยุด API server ที่รันแบบ background
Stop-Process -Id <PID>
Get-Process oscc-server | Stop-Process   # หรือปิดทั้งหมด
```

## ความปลอดภัย

- `postgres` (superuser) ใช้เฉพาะติดตั้ง/ดูแล — แอปห้ามใช้เด็ดขาด
- `pg_hba.conf` ของ EDB ตั้ง password auth (scram-sha-256) สำหรับ host connections แล้ว
- `listen_addresses` ควรเป็น `localhost` เท่านั้น — เครื่องอื่นใน LAN ต้องคุยผ่าน
  **API** (TLS) ไม่ใช่ต่อ 5432 ตรง ๆ
- รหัสผ่านทุกตัวเก็บใน password manager ของหน่วยงาน ไม่เข้า git/แชต/อีเมล
- Backup: `pg_dump` ผ่าน Scheduled Task + ซ้อม restore (แผนเต็มอยู่ใน M5)

## Troubleshooting

| อาการ | สาเหตุที่พบบ่อย | ทางแก้ |
|---|---|---|
| `password authentication failed` | `PGPASSWORD` ยังเป็นรหัสของอีกรole | ตั้ง `PGPASSWORD` ให้ตรงกับ `-U` ที่ใช้ |
| installer exit 1 ทันที | path มีช่องว่างไม่ได้ใส่ quote | ใส่ `"..."` ครอบ path ใน `-ArgumentList` |
| `psql` not recognized | ยังไม่เพิ่ม PATH หรือยังไม่เปิด terminal ใหม่ | เปิด terminal ใหม่ หรือใส่ full path |
| port 5432 ถูกใช้อยู่ | มี PostgreSQL อีกตัว | ตรวจ `Get-NetTCPConnection -LocalPort 5432` แล้วเปลี่ยน port |
| service start ไม่ขึ้น | ดู log ที่ `data\log\postgresql-*.log` | แก้ตาม error ใน log |

## ลำดับคำสั่งทั้งหมด (สรุป)

```powershell
winget show --id PostgreSQL.PostgreSQL.18 --exact
Invoke-WebRequest -Uri "https://get.enterprisedb.com/postgresql/postgresql-18.6-4-windows-x64.exe" -OutFile "$env:TEMP\pg18.exe" -UseBasicParsing
(Get-FileHash "$env:TEMP\pg18.exe" -Algorithm SHA256).Hash
Start-Process "$env:TEMP\pg18.exe" -Verb RunAs -Wait -ArgumentList @('--mode','unattended','--unattendedmodeui','none','--superpassword','<SUPER>','--serverport','5432','--servicename','postgresql-x64-18','--prefix','"C:\Program Files\PostgreSQL\18"','--datadir','"C:\Program Files\PostgreSQL\18\data"','--enable-components','server,commandlinetools')
[Environment]::SetEnvironmentVariable('Path', "$([Environment]::GetEnvironmentVariable('Path','User'));C:\Program Files\PostgreSQL\18\bin", 'User')
$env:PGPASSWORD='<SUPER>'
psql -U postgres -h 127.0.0.1 -c "CREATE ROLE oscc_app LOGIN PASSWORD '<APP>';"
psql -U postgres -h 127.0.0.1 -c "CREATE DATABASE oscc OWNER oscc_app;"
psql -U postgres -h 127.0.0.1 -c "CREATE DATABASE oscc_test OWNER oscc_app;"
Remove-Item Env:PGPASSWORD
cargo build -p oscc-server --example hash_password
$env:OSCC_DATABASE_URL='postgres://oscc_app:<APP>@127.0.0.1:5432/oscc'
Start-Process "target\debug\oscc-server.exe" -WorkingDirectory (Get-Location) -WindowStyle Hidden -PassThru
$hash = ('<USER_PASSWORD>' | & "target\debug\examples\hash_password.exe").Trim()
$env:PGPASSWORD='<APP>'
psql -U oscc_app -h 127.0.0.1 -d oscc -c "INSERT INTO users (username, display_name, role, password_hash) VALUES ('nurse.a','Nurse A','er_nurse','$hash');"
Remove-Item Env:PGPASSWORD
```
