//! Agent Mode Evaluation Module
//! 
//! This module provides agent performance evaluation framework,
//! metrics collection, and benchmarking tools for AI agent interactions.

pub mod evaluator;
pub mod metrics;
pub mod benchmark;
pub mod analysis;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvalConfig {
    pub enable_logging: bool,
    pub metrics_interval: u64,
    pub benchmark_enabled: bool,
    pub analysis_depth: u8,
}

impl Default for AgentEvalConfig {
    fn default() -> Self {
        Self {
            enable_logging: true,
            metrics_interval: 5000, // 5 seconds
            benchmark_enabled: false,
            analysis_depth: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub response_time: f64,
    pub accuracy_score: f64,
    pub success_rate: f64,
    pub error_count: u32,
    pub total_interactions: u32,
}

impl Default for AgentMetrics {
    fn default() -> Self {
        Self {
            response_time: 0.0,
            accuracy_score: 0.0,
            success_rate: 0.0,
            error_count: 0,
            total_interactions: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentEvaluator {
    #[allow(dead_code)]
    config: AgentEvalConfig,
    #[allow(dead_code)]
    metrics: AgentMetrics,
    session_data: HashMap<String, AgentMetrics>,
}

impl AgentEvaluator {
    pub fn new(config: AgentEvalConfig) -> Self {
        Self {
            config,
            metrics: AgentMetrics::default(),
            session_data: HashMap::new(),
        }
    }

    pub fn start_evaluation(&mut self, session_id: String) {
        self.session_data.insert(session_id, AgentMetrics::default());
    }

    pub fn record_interaction(&mut self, session_id: &str, response_time: f64, success: bool) {
        if let Some(metrics) = self.session_data.get_mut(session_id) {
            metrics.total_interactions += 1;
            metrics.response_time = (metrics.response_time + response_time) / 2.0;
            
            if success {
                metrics.success_rate = 
                    (metrics.success_rate * (metrics.total_interactions - 1) as f64 + 1.0) 
                    / metrics.total_interactions as f64;
            } else {
                metrics.error_count += 1;
                metrics.success_rate = 
                    (metrics.success_rate * (metrics.total_interactions - 1) as f64) 
                    / metrics.total_interactions as f64;
            }
        }
    }

    pub fn get_metrics(&self, session_id: &str) -> Option<&AgentMetrics> {
        self.session_data.get(session_id)
    }
}
