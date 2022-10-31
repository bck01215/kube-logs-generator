use kube_logs_generator::structures::{Condition, Pod, Pods};
use lazy_static::lazy_static;
use std::{
    env,
    fs::{create_dir, File, OpenOptions},
    io::{prelude::*, BufReader},
    path::Path,
    thread, time,
};
use tokio::task;
lazy_static! {
    static ref TOKEN: String = env::var("KUBE_TOKEN").unwrap_or_default();
    static ref URL: String = env::var("KUBE_HOST").unwrap_or_default();
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let conditions = set_conditions();
    println!("Evaluating pods for {:?}", conditions);
    let secs = time::Duration::from_secs(10);
    loop {
        let pods = get_pods_with_names(&conditions).await?;
        for pod in pods.iter() {
            let clone_pod = pod.clone();
            task::spawn(async {
                let _ = handle_pod(clone_pod).await;
            });
        }
        thread::sleep(secs);
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
        .get(URL.to_string() + "/api/v1/pods")
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
    let resp = client
        .get(URL.to_string() + &pod.metadata.self_link + "/log?timestamps=true")
        .header("Authorization", "Bearer ".to_string() + &TOKEN)
        .send()
        .await?;
    let _ = create_dir("./logs/".to_owned() + &pod.metadata.namespace);
    let file_name = &("./logs/".to_owned() + &pod.metadata.namespace + "/" + &pod.metadata.name + ".log");

    let text = resp.text().await?;
    let mut file = OpenOptions::new().write(true).append(true).create(true).read(true).open(file_name).unwrap();
    let chunks = text.split('\n');
    let cur_lines = lines_from_file(file_name);
    'while_loop: for chunk in chunks {
        let end_stamp_index = chunk.find('Z').unwrap_or(0);
        if chunk.len() < end_stamp_index + 2 {
            continue 'while_loop;
        }
        let timestamp = &chunk[..end_stamp_index + 1];
        let data = format!(
            r#"{{ "date": "{}", "msg": "{}", "pod": "{}", "namespace": "{}", "annotations": {:?} }}"#,
            timestamp,
            &chunk[end_stamp_index + 2..].trim_end().replace('"', "'"),
            pod.metadata.name,
            pod.metadata.namespace,
            pod.clone().metadata.annotations.unwrap_or_default()
        );
        if !cur_lines.contains(&data) {
            if let Err(e) = writeln!(file, "{}", data) {
                println!("Couldn't write to file: {}", e);
            }
        }
    }
    Ok(())
}

fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines().map(|l| l.expect("Could not parse line")).collect()
}
