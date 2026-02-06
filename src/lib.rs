use rtsyn_plugin::{PluginApi, PluginString};
use serde_json::Value;
use std::ffi::c_void;

const INPUTS: &[&str] = &["pre", "post"];
const OUTPUTS: &[&str] = &["i_syn"];

#[derive(Debug)]
struct PluginState {
    pre: f64,
    post: f64,
    output: f64,

    g_slow: f64,
    e_syn: f64,
    s_slow: f64,
    v_slow: f64,
    k_1x: f64,
    k_2x: f64,
    m_slow: f64,
    time_increment: f64,
}

impl Default for PluginState {
    fn default() -> Self {
        Self {
            pre: 0.0,
            post: 0.0,
            output: 0.0,
            g_slow: 0.046,
            e_syn: -1.92,
            s_slow: 1.0,
            v_slow: -1.74,
            k_1x: 0.74,
            k_2x: 0.007,
            m_slow: 0.0,
            time_increment: 0.0015,
        }
    }
}

extern "C" fn create(_: u64) -> *mut c_void {
    Box::into_raw(Box::new(PluginState::default())) as *mut c_void
}

extern "C" fn destroy(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle as *mut PluginState)) }
    }
}

extern "C" fn meta_json(_: *mut c_void) -> PluginString {
    let value = serde_json::json!({
        "name": "Slow Chemical Synapse",
        "default_vars": [
            ["g_slow", 0.046],
            ["e_syn", -1.92],
            ["s_slow", 1.0],
            ["v_slow", -1.74],
            ["k_1x", 0.74],
            ["k_2x", 0.007],
            ["time_increment", 0.0015]
        ]
    });
    PluginString::from_string(value.to_string())
}

extern "C" fn inputs_json(_: *mut c_void) -> PluginString {
    PluginString::from_string(serde_json::to_string(INPUTS).unwrap())
}

extern "C" fn outputs_json(_: *mut c_void) -> PluginString {
    PluginString::from_string(serde_json::to_string(OUTPUTS).unwrap())
}
extern "C" fn behavior_json(_handle: *mut c_void) -> PluginString {
    let behavior = serde_json::json!({
        "supports_start_stop": true,
        "supports_restart": true,
        "extendable_inputs": {"type": "none"},
        "loads_started": true
    });
    PluginString::from_string(behavior.to_string())
}

extern "C" fn display_schema_json(_handle: *mut c_void) -> PluginString {
    let schema = serde_json::json!({
        "outputs": ["i_syn"],
        "inputs": [],
        "variables": ["g_slow", "e_syn", "s_slow", "v_slow", "k_1x", "k_2x", "time_increment"]
    });
    PluginString::from_string(schema.to_string())
}


extern "C" fn set_config_json(handle: *mut c_void, data: *const u8, len: usize) {
    if handle.is_null() || data.is_null() {
        return;
    }

    let state = unsafe { &mut *(handle as *mut PluginState) };
    let slice = unsafe { std::slice::from_raw_parts(data, len) };

    if let Ok(Value::Object(map)) = serde_json::from_slice::<Value>(slice) {
        if let Some(v) = map.get("g_slow").and_then(|v| v.as_f64()) {
            state.g_slow = v;
        }
        if let Some(v) = map.get("e_syn").and_then(|v| v.as_f64()) {
            state.e_syn = v;
        }
        if let Some(v) = map.get("s_slow").and_then(|v| v.as_f64()) {
            state.s_slow = v;
        }
        if let Some(v) = map.get("v_slow").and_then(|v| v.as_f64()) {
            state.v_slow = v;
        }
        if let Some(v) = map.get("k_1x").and_then(|v| v.as_f64()) {
            state.k_1x = v;
        }
        if let Some(v) = map.get("k_2x").and_then(|v| v.as_f64()) {
            state.k_2x = v;
        }
        if let Some(v) = map.get("time_increment").and_then(|v| v.as_f64()) {
            state.time_increment = v;
        }
    }
}

extern "C" fn set_input(handle: *mut c_void, name: *const u8, len: usize, value: f64) {
    if handle.is_null() || name.is_null() {
        return;
    }

    let state = unsafe { &mut *(handle as *mut PluginState) };
    let key = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(name, len)) };

    match key {
        "pre" => state.pre = value,
        "post" => state.post = value,
        _ => {}
    }
}

extern "C" fn process(handle: *mut c_void, _tick: u64, _period: f64) {
    if handle.is_null() {
        return;
    }

    let s = unsafe { &mut *(handle as *mut PluginState) };

    let activation = s.k_1x * (1.0 - s.m_slow) / (1.0 + (s.s_slow * (s.v_slow - s.pre)).exp());
    let deactivation = s.k_2x * s.m_slow;

    s.m_slow += (activation - deactivation) * s.time_increment;
    s.output = s.g_slow * s.m_slow * (s.post - s.e_syn);
}

extern "C" fn get_output(handle: *mut c_void, name: *const u8, len: usize) -> f64 {
    if handle.is_null() || name.is_null() {
        return 0.0;
    }

    let state = unsafe { &*(handle as *mut PluginState) };
    let key = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(name, len)) };

    match key {
        "i_syn" => state.output,
        _ => 0.0,
    }
}

#[no_mangle]
pub extern "C" fn rtsyn_plugin_api() -> *const PluginApi {
    static API: PluginApi = PluginApi {
        create,
        destroy,
        meta_json,
        inputs_json,
        outputs_json,
        behavior_json: Some(behavior_json),
        display_schema_json: Some(display_schema_json),
        ui_schema_json: None,
        set_config_json,
        set_input,
        process,
        get_output,
    };
    &API
}
