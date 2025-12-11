use serde_json::{json, Value as JsonValue};

pub fn files_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "limit": {
                "type": "integer",
                "description": "Maximum number of files to return (default: 100)",
                "default": 100
            },
            "offset": {
                "type": "integer",
                "description": "Number of files to skip for pagination (default: 0)",
                "default": 0
            }
        }
    })
}

pub fn files_output_schema() -> JsonValue {
    json!({
        "type": "object",
        "description": "List of files in the system",
        "properties": {
            "items": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"},
                        "thumbnail": {"type": "string"},
                        "file_path": {"type": "string"},
                        "public_url": {"type": "string"},
                        "mime_type": {"type": "string"},
                        "file_size_bytes": {"type": ["integer", "null"]},
                        "ai_content": {"type": "boolean"},
                        "created_at": {"type": "string"}
                    }
                }
            },
            "count": {"type": "integer"}
        },
        "x-artifact-type": "table",
        "x-table-hints": {
            "columns": ["thumbnail", "file_path", "mime_type", "file_size_bytes", "ai_content", "created_at"],
            "sortable_columns": ["file_path", "mime_type", "file_size_bytes", "created_at"],
            "default_sort": {"column": "created_at", "order": "desc"},
            "filterable": true,
            "page_size": 25,
            "column_types": {
                "id": "string",
                "thumbnail": "thumbnail",
                "file_path": "string",
                "mime_type": "string",
                "file_size_bytes": "integer",
                "ai_content": "boolean",
                "created_at": "datetime",
                "public_url": "link"
            }
        }
    })
}
