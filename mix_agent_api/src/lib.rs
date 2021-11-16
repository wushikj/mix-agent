use access_json::JSONQuery;
use chrono::{DateTime, Local, NaiveDateTime, ParseResult, TimeZone};
use log::{error, info, warn};
use mix_agent_common::mix_config::{init_logger, MixConfig};
use mix_agent_common::{init_log, mix_config, post_log, GlobalConfig, LogLevel, Monitor, StripBom};

use evalexpr::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::{BTreeMap};
use std::error::Error;
use std::time::Duration;

const AGENT_NAME: &str = "mix_agent_api";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Result {
    name: String,
    auth: bool,
    targets: Vec<TargetResult>,
    success: bool,
    status: u16,
    message: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TargetResult {
    name: String,
    success: bool,
    status: u16,
    auth: bool,
    match_result: Vec<MatchResult>,
    message: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MatchResult {
    keyword: String,
    found: bool,
    matched: bool,
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ApiAgentConfig {
    #[serde(default)]
    name: String,
    #[serde(default = "default_cron")]
    cron: String,
    #[serde(default)]
    auth: Auth,
    #[serde(default)]
    target_source: TargetSource,
    #[serde(default)]
    targets: Vec<Target>,
}

impl Default for ApiAgentConfig {
    fn default() -> Self {
        ApiAgentConfig {
            name: "".to_string(),
            cron: default_cron(),
            auth: Default::default(),
            target_source: Default::default(),
            targets: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct Target {
    #[serde(default)]
    name: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    auth: bool,
    #[serde(default)]
    keywords: BTreeMap<String, String>,
    #[serde(default)]
    exclude: bool,
    #[serde(default)]
    condition: Option<Condition>,
}
#[derive(Debug, Deserialize, Default)]
struct Condition {
    #[serde(default)]
    url: String,
    #[serde(default)]
    vars: BTreeMap<String, String>,
    #[serde(default)]
    when: String,
}

#[derive(Debug, Deserialize, Default)]
struct Auth {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    keys: BTreeMap<String, String>,
    #[serde(default)]
    token: Token,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
struct Token {
    #[serde(default)]
    url: String,
    #[serde(default)]
    json_path: String,
    #[serde(default)]
    header: String,
}

#[derive(Debug, Deserialize, Default)]
struct TargetSource {
    #[serde(default)]
    ip: String,
    #[serde(default)]
    port: i32,
}

#[derive(Debug)]
struct Exp {
    marker: String,
    json_path: String,
    value: String,
    format: String,
}
///默认每15秒执行一次
fn default_cron() -> String {
    "*/15 * * * * ?".to_string()
}

impl Result {
    pub fn init() -> Result {
        init_logger(AGENT_NAME);
        info!("begin api data collect");
        Result {
            ..Default::default()
        }
    }
}

impl MixConfig for ApiAgentConfig {
    fn new() -> Self {
        ApiAgentConfig::default()
    }
}
impl Monitor for Result {
    fn collect(&self) {
        let global_config = mix_config::load::<GlobalConfig>("global");
        let agent_config = mix_config::load::<ApiAgentConfig>(AGENT_NAME);

        let mut tags = vec![];
        tags.push("agent-desc|api监控".to_owned());

        if agent_config.target_source.ip.trim().is_empty() {
            error!("{}", "target_source配置不正确");
            let log = init_log("agent", "未配置目标ip`target-source.ip`", LogLevel::Error, Box::new(tags.clone()), "", AGENT_NAME);
            post_log(&log);
            return;
        }

        info!("{:?}", global_config);
        info!("{:?}", agent_config);

        Self::begin(&agent_config.cron, || {
            let mut result = Result::default();
            result.name = agent_config.name.clone();
            result.auth = agent_config.auth.enabled;

            let auth_enabled = agent_config.auth.enabled;
            let token_url = &agent_config.auth.token.url;
            let token_json_path = &agent_config.auth.token.json_path;

            if auth_enabled {
                let client = reqwest::blocking::Client::new();
                let res = client.post(token_url).timeout(core::time::Duration::from_millis(global_config.timeout as u64)).json(&agent_config.auth.keys).send();
                match res {
                    Ok(res) => {
                        if res.status().is_success() {
                            let success = res.status().clone();
                            let status = res.status().as_u16();
                            let text = res.text().unwrap();
                            let token = get_token(text.as_str(), token_json_path);

                            let trs = check_targets(&global_config, &agent_config, token);
                            result.success = success.is_success();
                            result.status = status;
                            result.message = text.clone();
                            result.targets = trs;
                        } else {
                            let status = res.status().as_u16();
                            let text = res.text().unwrap();
                            error!("{} {}", agent_config.name, text);
                            result.status = status;
                            result.message = text.clone();
                        }
                    }
                    Err(e) => {
                        error!("{} {}", agent_config.name, e.to_string());
                        let msg = format!("{}:{}", agent_config.name, e.to_string());
                        let log = init_log("agent", msg.as_str(), LogLevel::Error, Box::new(tags.clone()), &result, AGENT_NAME);
                        post_log(&log);
                    }
                }
            } else {
                let trs = check_targets(&global_config, &agent_config, String::new());
                result.success = true;
                result.status = 200;
                result.message = String::new();
                result.targets = trs;
            }

            let log = init_log("api", "", LogLevel::Info, Box::new(tags.clone()), &result, AGENT_NAME);
            post_log(&log);
        });
    }
}

fn check_rule(token: &str, target: &Target) -> (Vec<MatchResult>, String, bool, u16) {
    let client = reqwest::blocking::Client::new();
    let url = target.url.as_str();
    let res = if target.auth {
        info!("token: {}", token);
        client.get(url).bearer_auth(token).send()
    } else {
        client.get(url).send()
    };

    let mut mrs: Vec<MatchResult> = vec![];
    let mut msg = String::new();
    let mut success = false;
    let mut status = 200;

    match res {
        Ok(res) => {
            if res.status().is_success() {
                success = true;
                status = res.status().as_u16();
                let content_r = res.text();
                match content_r {
                    Ok(content) => {
                        let content_str = content.strip_bom();
                        let json: Value = serde_json::from_str(content_str).unwrap();

                        for (k, v) in target.keywords.iter() {
                            let exp = parse_exp(v.to_string());
                            //info!("exp:{}", exp.json_path);
                            let query = JSONQuery::parse(exp.json_path.as_str()).unwrap();
                            let out = query.execute(&json).unwrap();
                            let mut mr = MatchResult::default();
                            mr.keyword = k.to_string();

                            if let Some(m) = out {
                                mr.found = true;
                                let mut find = String::new();
                                match m {
                                    Value::Bool(v) => find = v.to_string(),
                                    Value::Number(v) => find = v.to_string(),
                                    Value::String(v) => find = v.to_string(),
                                    _ => {
                                        warn!("不支持的匹配: {}", exp.json_path);
                                        continue;
                                    }
                                }

                                info!("keyword: {} --> {} = [{}]", k, v, find);
                                if exp.marker == "T" {
                                    let temp = find.clone();
                                    let (status, msg) = process_time_value(&exp, temp);
                                    mr.matched = status;
                                    mr.message = msg
                                } else {
                                    if find != exp.value {
                                        let msg = format!("实际: {}, 期待: {}, 匹配失败", find, exp.value);
                                        mr.message = msg.to_string();
                                        mr.matched = false;
                                        info!("{}", msg);
                                    } else {
                                        let msg = format!("实际: {}, 期待: {}, 匹配成功", find, exp.value);
                                        mr.matched = true;
                                        mr.message = msg.to_string();
                                        info!("{}", msg);
                                    }
                                }
                            } else {
                                warn!("未找到匹配：{}", exp.json_path);
                            }

                            mrs.push(mr);
                        }
                    }
                    Err(e) => {
                        error!("request error:{}", e);
                    }
                }
            } else {
                success = false;
                status = res.status().as_u16();
                msg = format!("{} {}", res.status(), res.text().unwrap());
                error!("{}", msg);
            }
        }
        Err(e) => {
            error!("Request url failed: {}", e.to_string());
            success = false;
            status = 400;
            msg = e.to_string();
        }
    }

    (mrs, msg, success, status)
}

fn parse_exp(exp: String) -> Exp {
    //N|.data.defaultFunction[0].sort|6.5
    let items: Vec<&str> = exp.split("|").collect();
    if items.len() < 3 {
        let msg = format!("表达式配置错: {}，正确格式为: T(类型标识)|jsonPath|V(匹配的值)", exp);
        warn!("{}", msg);
        panic!("{}", msg);
    }

    Exp {
        marker: items.get(0).value().to_string(),
        json_path: items.get(1).value().to_string(),
        value: items.get(2).value().trim().to_string(),
        format: if items.len() == 4 {
            items.get(3).value().to_string()
        } else {
            String::new()
        },
    }
}

trait ValueGetter<T> {
    fn value(self) -> T;
}

impl<T> ValueGetter<T> for Option<T> {
    fn value(self) -> T {
        match self {
            None => None.expect("called `Result::unwrap()` on an `Err` value"),
            Some(v) => v.into(),
        }
    }
}

#[test]
fn test_get_value_some() {
    let none: Option<&str> = Some("abc");
    let r = none.value();
    assert_eq!(r, "abc");
}

#[test]
fn test_get_value_none() {
    let none: Option<&str> = None;
    let r = none.value();
}

#[test]
fn test_parse_exp() {
    parse_exp(String::from(".data.defaultFunction[0].sort|6.5"));
}

#[test]
fn test_parse_date() {
    let dt = Local.ymd(2020, 3, 6).and_hms(12, 0, 9);
    let t2: ParseResult<DateTime<Local>> = Local.datetime_from_str("2020-03-06T12:00:09", "%Y-%m-%dT%H:%M:%S");
    println!("{:?}", t2);
    assert_eq!(dt, t2.unwrap());

    t2.unwrap().format("%Y-%m-%dT%H:%M:%S").to_string();

    let a = Local.datetime_from_str("2014-11-28 12:00:09", "%Y-%m-%d %H:%M:%S");
    println!("{:?}", a);
}

fn parse_time(input: &String) -> i64 {
    let r = input.to_lowercase();
    let mut result = 300;
    if r.ends_with("d") {
        result = r.trim_end_matches("d").parse::<i64>().unwrap() * 24 * 60 * 60;
    } else if r.ends_with("h") {
        result = r.trim_end_matches("h").parse::<i64>().unwrap() * 60 * 60;
    } else if r.ends_with("m") {
        result = r.trim_end_matches("m").parse::<i64>().unwrap() * 60;
    } else if r.ends_with("s") {
        result = r.trim_end_matches('s').parse::<i64>().unwrap();
    }

    result
}

fn get_token(text: &str, token_json_path: &str) -> String {
    let json: Value = serde_json::from_str(text).unwrap();
    let selector = JSONQuery::parse(token_json_path).unwrap();
    let tokens = selector.execute(&json).unwrap();
    let token = tokens.unwrap();
    token.as_str().unwrap().to_string()
}

fn process_time_value(exp: &Exp, m: String) -> (bool, String) {
    //let time = chrono::Local.ymd(2020, 6, 29).and_hms(17, 25, 0);
    let mut time = Local::now().naive_local();
    let mut target_time = Local::now().naive_local();

    if exp.format == "%Y-%m-%d" {
        let mut time_string = m;
        time_string.push_str(" 00:00:00");

        let mut current_date = time.format("%Y-%m-%d").to_string();
        current_date.push_str(" 00:00:00");

        time = NaiveDateTime::parse_from_str(current_date.as_str(), "%Y-%m-%d %H:%M:%S").unwrap();
        target_time = NaiveDateTime::parse_from_str(time_string.as_str(), "%Y-%m-%d %H:%M:%S").unwrap();
        //target_time = NaiveDateTime::parse_from_str("2021-04-23 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    } else {
        target_time = NaiveDateTime::parse_from_str(m.as_str(), exp.format.as_str()).unwrap();
    }

    let duration = time.signed_duration_since(target_time);
    let diff = duration.num_seconds();
    let threshold = parse_time(&exp.value);
    if diff >= threshold {
        let msg = format!("当前时间: {}, 目标时间: {}, 差值：{}s >= 阀值：{}s({}), 不符合时间范围", time, target_time, diff, threshold, &exp.value);
        info!("{}", msg);
        (false, msg)
    } else {
        let msg = format!("当前时间: {}, 目标时间: {}, 差值：{}s < 阀值：{}s({}), 符合时间范围", time, target_time, diff, threshold, &exp.value);
        info!("{}", msg);
        (true, msg)
    }
}

fn check_targets(global_config: &GlobalConfig, agent_config: &ApiAgentConfig, token: String) -> Vec<TargetResult> {
    let mut trs: Vec<TargetResult> = vec![];
    for target in agent_config.targets.iter() {
        if target.exclude {
            info!("{} -> {}(exclude=true)", agent_config.name, target.name);
            continue;
        }

        if target.condition.is_some() {
            if check_condition(&target, &global_config) {
                create_target_result(token.as_str(), &mut trs, target, &global_config);
            } else {
                info!("{}", "when计算结果为false，不执行该target")
            }
        } else {
            create_target_result(token.as_str(), &mut trs, target, &global_config);
        }
    }
    trs
}

fn create_target_result(token: &str, trs: &mut Vec<TargetResult>, target: &Target, _global_config: &GlobalConfig) {
    let mut target_result = TargetResult {
        name: target.name.as_str().to_string(),
        success: false,
        status: 200,
        auth: target.auth,
        match_result: vec![],
        message: String::new(),
    };

    let (mrs, msg, success, status) = check_rule(token, target);
    target_result.match_result = mrs;
    target_result.message = msg;
    target_result.success = success;
    target_result.status = status;
    trs.push(target_result);
}

fn check_condition(target: &Target, global_config: &GlobalConfig) -> bool {
    if let Some(ref condition) = target.condition {
        let _url = &condition.url;
        info!("when条件表达式为: {}", &condition.when);

        let data = get_response_data(_url.to_string(), global_config);
        match data {
            Ok(content) => {
                let content_str = content.strip_bom();
                let mut context = HashMapContext::new();
                let json: Value = serde_json::from_str(content_str).unwrap();
                for (k, v) in condition.vars.iter() {
                    let query = JSONQuery::parse(v.as_str()).unwrap();
                    if let Some(m) = query.execute(&json).unwrap() {
                        let mut find = evalexpr::Value::from(());
                        match m {
                            Value::Bool(v) => {
                                find = evalexpr::Value::Boolean(v);
                            }
                            Value::Number(v) => {
                                if v.is_i64() {
                                    find = evalexpr::Value::Int(v.as_i64().unwrap());
                                } else if v.is_f64() || v.is_u64() {
                                    find = evalexpr::Value::Float(v.as_f64().unwrap());
                                } else {
                                    error!("不支持的数据类型");
                                }
                            }
                            Value::String(v) => {
                                find = evalexpr::Value::String(v);
                            }
                            _ => {
                                warn!("不支持的匹配: {}", v);
                                continue;
                            }
                        }
                        info!(" var: {} --> {} = [{:?}]", k, v, find);

                        context.set_value(k.to_string(), find);
                    }
                }

                let eval_result = eval_boolean_with_context(&condition.when, &context);
                return match eval_result {
                    Ok(result) => {
                        info!("when条件计算结果: {}", result);
                        result
                    }
                    Err(e) => {
                        error!("{}", e);
                        false
                    }
                };
            }
            Err(e) => {
                error!("{}", e);
            }
        }
        return false;
    };

    return false;
}

fn get_response_data(url: String, global_config: &GlobalConfig) -> std::result::Result<String, Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let response = client.get(url.as_str()).timeout(Duration::from_millis(global_config.timeout)).send();
    match response {
        Ok(res) => {
            let text = res.text()?;
            Ok(text)
        }
        Err(e) => Err(Box::from(e)),
    }
}
