use chrono::{DateTime, Local};

use crate::api::{self, BalanceResponse};
use crate::platform::{self, DayUsage};

#[derive(Debug, Clone)]
pub struct UsageStats {
    pub tokens_today: u64,
    pub tokens_month: u64,
    pub tokens_all_time: u64,
}

pub struct App {
    pub balance: Option<BalanceResponse>,
    pub usage: Option<UsageStats>,
    pub balance_remaining_usd: Option<String>,
    pub last_balance_update: Option<DateTime<Local>>,
    pub last_usage_update: Option<DateTime<Local>>,
    pub balance_error: Option<String>,
    pub usage_error: Option<String>,
    pub is_loading_balance: bool,
    pub is_loading_usage: bool,
    balance_client: Option<api::DeepSeekClient>,
    platform_client: Option<platform::PlatformClient>,
    cached_all_time: u64,
}

impl App {
    pub fn new(api_key: Option<String>, platform_token: Option<String>) -> Self {
        let balance_client = api_key.map(api::DeepSeekClient::new);
        let platform_client = platform_token.map(platform::PlatformClient::new);

        Self {
            balance: None,
            usage: None,
            balance_remaining_usd: None,
            last_balance_update: None,
            last_usage_update: None,
            balance_error: None,
            usage_error: None,
            is_loading_balance: false,
            is_loading_usage: false,
            balance_client,
            platform_client,
            cached_all_time: 0,
        }
    }

    pub fn has_platform(&self) -> bool {
        self.platform_client.is_some()
    }

    pub async fn refresh_balance(&mut self) {
        let Some(ref client) = self.balance_client else {
            self.balance_error = Some("DEEPSEEK_API_KEY not set".into());
            self.last_balance_update = Some(Local::now());
            return;
        };

        self.is_loading_balance = true;
        self.balance_error = None;

        match client.fetch_balance().await {
            Ok(balance) => {
                self.balance = Some(balance);
                self.balance_error = None;
                self.last_balance_update = Some(Local::now());
            }
            Err(e) => {
                self.balance_error = Some(format!("{e:#}"));
                self.last_balance_update = Some(Local::now());
            }
        }

        self.is_loading_balance = false;
    }

    pub async fn refresh_usage(&mut self) {
        let Some(ref client) = self.platform_client else {
            return;
        };

        self.is_loading_usage = true;
        self.usage_error = None;

        let mut all_time_from_summary: u64 = 0;

        match client.get_user_summary().await {
            Ok(summary) => {
                let total = summary
                    .normal_wallets
                    .first()
                    .map(|w| w.balance.clone())
                    .unwrap_or_default();
                self.balance_remaining_usd = Some(total);

                all_time_from_summary = summary
                    .total_usage
                    .as_ref()
                    .and_then(|v| v.as_u64())
                    .filter(|&n| n > 0)
                    .or_else(|| {
                        summary
                            .monthly_token_usage
                            .as_ref()
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<u64>().ok())
                    })
                    .unwrap_or(0);
            }
            Err(e) => {
                self.usage_error = Some(format!("summary: {e:#}"));
            }
        }

        match client.get_usage_amount(None, None).await {
            Ok(data) => {
                let today_str = Local::now().format("%Y-%m-%d").to_string();

                let today_tokens: u64 = data
                    .days
                    .iter()
                    .find(|d| d.date == today_str)
                    .map(day_total_tokens)
                    .unwrap_or(0);

                let month_tokens: u64 = data.days.iter().map(day_total_tokens).sum();

                if all_time_from_summary > self.cached_all_time {
                    self.cached_all_time = all_time_from_summary;
                }
                if month_tokens > self.cached_all_time {
                    self.cached_all_time = month_tokens;
                }

                self.usage = Some(UsageStats {
                    tokens_today: today_tokens,
                    tokens_month: month_tokens,
                    tokens_all_time: self.cached_all_time,
                });

                if self.usage_error.as_deref() == Some("") {
                    self.usage_error = None;
                }
            }
            Err(e) => {
                let prev = self.usage_error.take().unwrap_or_default();
                self.usage_error = Some(format!("{prev} amount: {e:#}"));
            }
        }

        self.last_usage_update = Some(Local::now());
        self.is_loading_usage = false;
    }
}

fn day_total_tokens(day: &DayUsage) -> u64 {
    day.data
        .iter()
        .flat_map(|m| &m.usage)
        .filter_map(|u| u.amount.parse::<u64>().ok())
        .sum()
}
