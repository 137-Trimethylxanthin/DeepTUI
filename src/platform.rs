use chrono::{Datelike, Utc};
use serde::Deserialize;

const PLATFORM_BASE: &str = "https://platform.deepseek.com";

#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    pub data: ApiData<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiData<T> {
    pub biz_data: T,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct UserSummaryBiz {
    pub current_token: Option<serde_json::Value>,
    pub total_usage: Option<serde_json::Value>,
    pub monthly_usage: Option<serde_json::Value>,
    pub monthly_token_usage: Option<serde_json::Value>,
    pub total_available_token_estimation: Option<serde_json::Value>,
    pub normal_wallets: Vec<WalletInfo>,
    pub bonus_wallets: Vec<WalletInfo>,
    pub monthly_costs: Vec<CostInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct WalletInfo {
    pub currency: String,
    pub balance: String,
    pub token_estimation: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct CostInfo {
    pub currency: String,
    pub amount: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageAmountData {
    #[allow(dead_code)]
    pub total: Vec<ModelTotal>,
    pub days: Vec<DayUsage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelTotal {
    #[allow(dead_code)]
    pub model: String,
    pub usage: Vec<UsageEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DayUsage {
    pub date: String,
    pub data: Vec<ModelTotal>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageEntry {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub usage_type: String,
    pub amount: String,
}

#[derive(Debug, Clone)]
pub struct PlatformClient {
    token: String,
    use_bearer: bool,
    client: reqwest::Client,
}

impl PlatformClient {
    pub fn new(token: String) -> Self {
        let use_bearer = !token.contains('=') && token.len() > 20;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("failed to build HTTP client");

        Self {
            token,
            use_bearer,
            client,
        }
    }

    fn apply_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.use_bearer {
            req.header("Authorization", format!("Bearer {}", self.token))
        } else {
            req.header("Cookie", &self.token)
        }
    }

    pub async fn get_user_summary(&self) -> Result<UserSummaryBiz, anyhow::Error> {
        let url = format!("{}/api/v0/users/get_user_summary", PLATFORM_BASE);
        let resp = self.apply_auth(self.client.get(&url)).send().await?;
        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            anyhow::bail!("Platform API error ({}): {}", status.as_u16(), trunc(&body));
        }

        let summary: ApiResponse<UserSummaryBiz> = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("parse error: {e}"))?;

        Ok(summary.data.biz_data)
    }

    pub async fn get_usage_amount(
        &self,
        year: Option<i32>,
        month: Option<u32>,
    ) -> Result<UsageAmountData, anyhow::Error> {
        let now = Utc::now();
        let y = year.unwrap_or_else(|| now.year());
        let m = month.unwrap_or_else(|| now.month());

        let url = format!("{}/api/v0/usage/amount", PLATFORM_BASE);
        let resp = self
            .apply_auth(
                self.client
                    .get(&url)
                    .query(&[("year", y.to_string()), ("month", m.to_string())]),
            )
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            anyhow::bail!("Platform API error ({}): {}", status.as_u16(), trunc(&body));
        }

        let amount: ApiResponse<UsageAmountData> = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("parse error: {e}"))?;

        Ok(amount.data.biz_data)
    }
}

fn trunc(s: &str) -> String {
    s.chars().take(200).collect()
}
