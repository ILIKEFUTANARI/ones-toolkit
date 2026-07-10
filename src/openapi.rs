use crate::api::{
    ActivityResponse, GroupsResponse, IssueResponse, ViewResponse, ViewsResponse,
    WorkbenchIssuePage,
};
use serde_json::{Map, Value, json};

pub fn document() -> Value {
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "ONES Issue Wrapper",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "security": [{ "onesEmail": [], "onesPassword": [] }],
        "paths": {
            "/api/workbench/views": {
                "get": operation("ViewsResponse", vec![])
            },
            "/api/workbench/views/{view_uuid}": {
                "get": operation("ViewResponse", vec![path_parameter("view_uuid")])
            },
            "/api/workbench/views/{view_uuid}/groups": {
                "get": operation("GroupsResponse", vec![path_parameter("view_uuid")])
            },
            "/api/workbench/views/{view_uuid}/issues": {
                "get": operation("WorkbenchIssuePage", vec![
                    path_parameter("view_uuid"),
                    query_parameter("group_value", "string", false),
                    query_parameter("cursor", "string", false),
                    query_parameter("limit", "integer", false),
                    redact_parameter(),
                ])
            },
            "/api/issues/by-display-id/{display_id}": {
                "get": operation("IssueResponse", vec![
                    path_parameter("display_id"),
                    redact_parameter(),
                ])
            },
            "/api/issues/{issue_uuid}": {
                "get": operation("IssueResponse", vec![
                    path_parameter("issue_uuid"),
                    redact_parameter(),
                ])
            },
            "/api/issues/{issue_uuid}/activity": {
                "get": operation("ActivityResponse", vec![
                    path_parameter("issue_uuid"),
                    query_parameter("cursor", "string", false),
                    query_parameter("limit", "integer", false),
                    redact_parameter(),
                ])
            }
        },
        "components": {
            "securitySchemes": {
                "onesEmail": header_auth("x-ones-email"),
                "onesPassword": header_auth("x-ones-password"),
            },
            "schemas": response_schemas(),
        }
    })
}

fn operation(schema: &str, parameters: Vec<Value>) -> Value {
    json!({
        "parameters": parameters,
        "responses": {
            "200": {
                "description": "Successful response",
                "content": {
                    "application/json": {
                        "schema": { "$ref": format!("#/components/schemas/{schema}") }
                    }
                }
            }
        }
    })
}

fn path_parameter(name: &str) -> Value {
    query_parameter(name, "string", true)
        .as_object()
        .cloned()
        .map(|mut parameter| {
            parameter.insert("in".to_string(), json!("path"));
            Value::Object(parameter)
        })
        .unwrap_or(Value::Null)
}

fn query_parameter(name: &str, schema_type: &str, required: bool) -> Value {
    json!({
        "name": name,
        "in": "query",
        "required": required,
        "schema": { "type": schema_type }
    })
}

fn redact_parameter() -> Value {
    json!({
        "name": "redact",
        "in": "query",
        "required": false,
        "schema": { "type": "boolean", "default": true },
        "description": "Redact structured user identities; defaults to true"
    })
}

fn header_auth(name: &str) -> Value {
    json!({ "type": "apiKey", "in": "header", "name": name })
}

fn response_schemas() -> Value {
    let mut schemas = Map::new();
    insert_schema::<ViewsResponse>(&mut schemas, "ViewsResponse");
    insert_schema::<ViewResponse>(&mut schemas, "ViewResponse");
    insert_schema::<GroupsResponse>(&mut schemas, "GroupsResponse");
    insert_schema::<WorkbenchIssuePage>(&mut schemas, "WorkbenchIssuePage");
    insert_schema::<IssueResponse>(&mut schemas, "IssueResponse");
    insert_schema::<ActivityResponse>(&mut schemas, "ActivityResponse");
    Value::Object(schemas)
}

fn insert_schema<T: schemars::JsonSchema>(schemas: &mut Map<String, Value>, name: &str) {
    let schema = schemars::schema_for!(T);
    let mut schema = serde_json::to_value(schema).unwrap();
    let definitions = schema
        .as_object_mut()
        .and_then(|schema| schema.remove("$defs"))
        .and_then(|definitions| definitions.as_object().cloned())
        .unwrap_or_default();
    rewrite_definition_refs(&mut schema);
    schemas.insert(name.to_string(), schema);
    for (definition_name, mut definition) in definitions {
        rewrite_definition_refs(&mut definition);
        schemas.entry(definition_name).or_insert(definition);
    }
}

fn rewrite_definition_refs(value: &mut Value) {
    match value {
        Value::Object(object) => {
            let rewritten_reference = object
                .get("$ref")
                .and_then(Value::as_str)
                .and_then(|reference| reference.strip_prefix("#/$defs/"))
                .map(|name| format!("#/components/schemas/{name}"));
            if let Some(reference) = rewritten_reference {
                object.insert("$ref".to_string(), Value::String(reference));
            }
            for nested in object.values_mut() {
                rewrite_definition_refs(nested);
            }
        }
        Value::Array(values) => {
            for nested in values {
                rewrite_definition_refs(nested);
            }
        }
        _ => {}
    }
}
