use ones::{
    api::{
        ActivityResponse, AttachmentsResponse, CodeLinksResponse, CreateBranchResponse,
        GroupsResponse, IssueResponse, LegacyTaskInfoResponse, TransitionsResponse, ViewResponse,
        ViewsResponse, WorkbenchIssuePage,
    },
    models::PageInfo,
};
use serde::Serialize;

pub fn print_json<T: Serialize>(value: &T) -> ones::AppResult<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

pub fn print_views(value: &ViewsResponse, json: bool) -> ones::AppResult<()> {
    if json {
        return print_json(value);
    }
    println!("UUID\tNAME");
    for view in &value.views {
        println!("{}\t{}", view.uuid, view.name);
    }
    Ok(())
}

pub fn print_groups(value: &GroupsResponse, json: bool) -> ones::AppResult<()> {
    if json {
        return print_json(value);
    }
    println!("VALUE\tLABEL\tCOUNT");
    for group in &value.groups {
        println!(
            "{}\t{}\t{}",
            group.value.as_deref().unwrap_or("-"),
            group.label,
            group.total_count
        );
    }
    Ok(())
}

pub fn print_issues(value: &WorkbenchIssuePage, json: bool) -> ones::AppResult<()> {
    if json {
        return print_json(value);
    }
    println!("DISPLAY_ID\tSTATUS\tTITLE\tPROJECT\tASSIGNEE\tUUID");
    for issue in &value.issues {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            issue.display_id.as_deref().unwrap_or("-"),
            issue
                .status
                .as_ref()
                .map(|status| status.name.as_str())
                .unwrap_or("-"),
            issue.title.as_deref().unwrap_or("-"),
            issue.project.as_deref().unwrap_or("-"),
            issue.assignee.as_deref().unwrap_or("-"),
            issue.uuid
        );
    }
    print_page_info(&value.page_info);
    Ok(())
}

pub fn print_attachments(value: &AttachmentsResponse, json: bool) -> ones::AppResult<()> {
    if json {
        return print_json(value);
    }
    println!("UUID\tNAME\tMIME\tSIZE\tDOWNLOAD_URL");
    for attachment in &value.attachments {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            attachment.uuid,
            attachment.name,
            attachment.mime.as_deref().unwrap_or("-"),
            attachment
                .size
                .map(|size| size.to_string())
                .as_deref()
                .unwrap_or("-"),
            attachment.download_url.as_deref().unwrap_or("-")
        );
    }
    print_page_info(&value.page_info);
    Ok(())
}

pub fn print_activity(value: &ActivityResponse, json: bool) -> ones::AppResult<()> {
    if json {
        return print_json(value);
    }
    println!("TIME\tACTOR\tSUMMARY\tTEXT");
    for entry in &value.activity {
        println!(
            "{}\t{}\t{}\t{}",
            entry.time.as_deref().unwrap_or("-"),
            entry.actor.as_deref().unwrap_or("-"),
            entry.summary.as_deref().unwrap_or("-"),
            entry.text.as_deref().unwrap_or("-")
        );
    }
    print_page_info(&value.page_info);
    Ok(())
}

pub fn print_details<T: Serialize>(value: &T, _json: bool) -> ones::AppResult<()> {
    print_json(value)
}

pub fn print_code_links(value: &CodeLinksResponse, json: bool) -> ones::AppResult<()> {
    print_details(value, json)
}

pub fn print_view(value: &ViewResponse, json: bool) -> ones::AppResult<()> {
    print_details(value, json)
}

pub fn print_issue(value: &IssueResponse, json: bool) -> ones::AppResult<()> {
    print_details(value, json)
}

pub fn print_legacy_task_info(value: &LegacyTaskInfoResponse, json: bool) -> ones::AppResult<()> {
    print_details(value, json)
}

pub fn print_transitions(value: &TransitionsResponse, json: bool) -> ones::AppResult<()> {
    print_details(value, json)
}

pub fn print_branch(value: &CreateBranchResponse, json: bool) -> ones::AppResult<()> {
    print_details(value, json)
}

fn print_page_info(page_info: &PageInfo) {
    if page_info.has_next_page {
        println!(
            "next_cursor={}{}",
            page_info.next_cursor.as_deref().unwrap_or("-"),
            page_info
                .total_count
                .map(|total| format!(" total_count={total}"))
                .unwrap_or_default()
        );
    }
}
