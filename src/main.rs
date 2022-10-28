use kube_logs_generator::structures::{Condition, Pod, Pods};
use lazy_static::lazy_static;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::Write;
use std::{thread, time};
use tokio::task;

lazy_static! {
    static ref TOKEN: String = env::var("KUBE_TOKEN").unwrap_or_default();
    static ref URL: String = env::var("KUBE_HOST").unwrap_or_default();
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let conditions = set_conditions();
    let mut all_pods: Vec<Pod> = Vec::new();
    let ten_secs = time::Duration::from_secs(60);
    loop {
        let pods = get_pods_with_names(&conditions).await?;
        for pod in pods.iter() {
            let pod_clone = pod.clone();
            if !all_pods.contains(pod) {
                all_pods.push(pod.clone());
                task::spawn(async {
                    let _ = handle_pod(pod_clone).await;
                });
            }
        }
        thread::sleep(ten_secs);
    }
}

fn set_conditions() -> Vec<Condition> {
    let mut conditions: Vec<Condition> = Vec::new();
    let conditions_env = env::var("CONDITIONS").unwrap_or_default();
    let conditions_arr = conditions_env.split(',');
    for condition_raw in conditions_arr {
        let cond_matching = !condition_raw.contains('!');
        let index = if cond_matching {
            let index = condition_raw.find('=').unwrap_or_default();
            (index, index + 1)
        } else {
            let index = condition_raw.find('!').unwrap_or_default();
            (index, index + 2)
        };
        let key = &condition_raw[..index.0];
        let value = &condition_raw[index.1..];
        let condition = Condition {
            key: key.to_string(),
            value: value.to_string(),
            matching: cond_matching,
        };

        conditions.push(condition);
    }
    conditions
}

async fn get_pods_with_names(conditions: &Vec<Condition>) -> Result<Vec<Pod>, Box<dyn std::error::Error>> {
    let mut pods: Vec<Pod> = Vec::new();
    let client = reqwest::Client::new();
    let resp = client
        .get("https://okd-cluster.liberty.edu".to_string() + "/api/v1/pods")
        .header("Authorization", "Bearer ".to_string() + &TOKEN)
        .send()
        .await?
        .json::<Pods>()
        .await?;
    'outer: for pod in resp.items.into_iter() {
        for cond in conditions {
            if (!cond.matching && pod.clone().metadata.labels.unwrap_or_default().get(&cond.key).unwrap_or(&"".to_string()) == &cond.value)
                || (cond.matching && pod.clone().metadata.labels.unwrap_or_default().get(&cond.key).unwrap_or(&"".to_string()) != &cond.value)
            {
                continue 'outer;
            }
        }
        pods.push(pod);
    }
    Ok(pods)
}

async fn handle_pod(pod: Pod) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut resp = client
        .get("https://okd-cluster.liberty.edu".to_string() + &pod.metadata.self_link + "/log?timestamps=true&follow=true&tailLines=0")
        .header("Authorization", "Bearer ".to_string() + &TOKEN)
        .send()
        .await?;
    let _ = fs::create_dir("./logs/".to_owned() + &pod.metadata.namespace);
    let file_name = &("./logs/".to_owned() + &pod.metadata.namespace + "/" + &pod.metadata.name + ".log");
    'while_loop: while let Some(chunk) = resp.chunk().await? {
        let s = match std::str::from_utf8(&chunk) {
            Ok(v) => v,
            Err(_e) => continue 'while_loop,
        };
        let end_stamp_index = s.find('Z').unwrap_or(0);
        if s.len() < end_stamp_index + 2 {
            continue 'while_loop;
        }
        let timestamp = &s[..end_stamp_index + 1];
        let data = format!(
            r#"{{ "date": "{}", "msg": "{}", "pod": "{}", "namespace": "{}", "annotations": {:?} }}"#,
            timestamp,
            &s[end_stamp_index + 2..].trim_end().replace('"', "'"),
            pod.metadata.name,
            pod.metadata.namespace,
            pod.clone().metadata.annotations.unwrap_or_default()
        );
        let mut file = OpenOptions::new().write(true).append(true).create(true).open(file_name).unwrap();

        if let Err(e) = writeln!(file, "{}", data) {
            println!("Couldn't write to file: {}", e);
        }
    }
    Ok(())
}
