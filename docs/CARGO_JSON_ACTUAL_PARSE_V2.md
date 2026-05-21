# CARGO_JSON_ACTUAL_PARSE_V2

`cargo_json_actual_parse_v2` parses actual cargo JSON lines and reports artifact and compiler-message counts separately from fixtures.

Malformed lines and parse failures are tracked in dedicated reports so cargo JSON progress cannot be misread as acceptance.
