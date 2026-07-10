use crate::ones::field_catalog::CoreFields;

pub(crate) fn test_core_fields() -> CoreFields {
    CoreFields {
        display_id: "display".into(),
        title: "title".into(),
        issue_type: "type".into(),
        status: "status".into(),
        project: "project".into(),
        creator: "creator".into(),
        assignee: "assignee".into(),
        watchers: "watchers".into(),
        priority: "priority".into(),
        sprint: "sprint".into(),
        modules: "modules".into(),
        due_date: "due".into(),
        created_at: "created".into(),
        updated_at: "updated".into(),
        description: "description".into(),
        parent: Some("parent".into()),
        children: Some("children".into()),
    }
}
