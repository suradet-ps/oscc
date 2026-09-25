//! Thai labels for machine names (UI strings live in `app/`).

/// Case status.
pub fn status_label(status: &str) -> &'static str {
    match status {
        "intake" => "รับแจ้ง",
        "active" => "กำลังดำเนินการ",
        "follow_up" => "ติดตาม",
        "closed" => "ปิดเคส",
        _ => "ไม่ทราบสถานะ",
    }
}

/// Incident type.
pub fn incident_label(incident: &str) -> &'static str {
    match incident {
        "sexual_assault" => "ล่วงละเมิดทางเพศ",
        "domestic_violence" => "ความรุนแรงในครอบครัว",
        "child_abuse" => "ทำร้ายเด็ก",
        "physical_assault" => "ทำร้ายร่างกาย",
        "trafficking" => "ค้ามนุษย์",
        "other" => "อื่น ๆ",
        _ => "ไม่ทราบประเภท",
    }
}

/// Risk level.
pub fn risk_label(risk: &str) -> &'static str {
    match risk {
        "low" => "ต่ำ",
        "medium" => "ปานกลาง",
        "high" => "สูง",
        "critical" => "วิกฤต",
        _ => "ไม่ทราบระดับ",
    }
}

/// Owning department.
pub fn department_label(department: &str) -> &'static str {
    match department {
        "emergency" => "ฉุกเฉิน",
        "forensic" => "นิติเวช",
        "social_work" => "สังคมสงเคราะห์",
        "psychology" => "จิตวิทยา",
        "oscc" => "OSCC",
        "it" => "ไอที",
        _ => "ไม่ทราบหน่วยงาน",
    }
}

/// Risk chip colour modifier.
pub fn risk_class(risk: &str) -> &'static str {
    match risk {
        "critical" | "high" => "chip--risk-high",
        "medium" => "chip--risk-medium",
        _ => "chip--risk-low",
    }
}

/// Status chip colour modifier.
pub fn status_class(status: &str) -> &'static str {
    match status {
        "intake" => "chip--status-intake",
        "active" => "chip--status-active",
        "follow_up" => "chip--status-followup",
        _ => "chip--status-closed",
    }
}
