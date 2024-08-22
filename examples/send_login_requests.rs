use std::collections::HashMap;

use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let mut login_params = HashMap::new();
    login_params.insert("username", "3a385877-1460-49c9-81fc-00f9cdf7611f");
    login_params.insert("password", "732c7dea-8a98-4102-a5db-ae977f3c9d54");

    _multithread(&client, &login_params).await?;

    Ok(())
}
async fn _singlethread(
    client: &reqwest::Client,
    login_params: &HashMap<&str, &str>,
) -> anyhow::Result<()> {
    for _ in 0..100 {
        let resp = client
            .post("http://localhost:8080/login")
            .form(login_params)
            .send()
            .await?
            .error_for_status()?;
        assert_eq!(200, resp.status().as_u16());
    }

    Ok(())
}

async fn _multithread(
    client: &reqwest::Client,
    login_params: &HashMap<&str, &str>,
) -> anyhow::Result<()> {
    let mut join_set = JoinSet::new();
    for _ in 0..100 {
        let client = client.clone();
        let login_params = login_params.clone();
        join_set.spawn(
            client
                .post("http://localhost:8080/login")
                .form(&login_params)
                .send(),
        );
    }
    while let Some(resp) = join_set.join_next().await {
        let resp = resp??.error_for_status()?;
        assert_eq!(200, resp.status().as_u16());
    }

    Ok(())
}
