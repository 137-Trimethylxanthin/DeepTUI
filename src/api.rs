use serde::Deserialize;

const DEEPSEEK_API_BASE: &str = "https://api.deepseek.com";

#[derive(Debug, Clone, Deserialize)]
pub struct BalanceResponse {
    pub is_available: bool,
    pub balance_infos: Vec<BalanceInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BalanceInfo {
    pub currency: String,
    pub total_balance: String,
    pub granted_balance: String,
    pub topped_up_balance: String,
}

#[derive(Debug, Clone)]
pub struct DeepSeekClient {
    api_key: String,
    client: reqwest::Client,
}

impl DeepSeekClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("failed to build HTTP client");

        Self { api_key, client }
    }

    pub async fn fetch_balance(&self) -> Result<BalanceResponse, anyhow::Error> {
        let url = format!("{}/user/balance", DEEPSEEK_API_BASE);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            anyhow::bail!(
                "API error ({}): {}",
                status.as_u16(),
                body.trim()
            );
        }

        let balance: BalanceResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("failed to parse response: {e}"))?;

        Ok(balance)
    }
}
