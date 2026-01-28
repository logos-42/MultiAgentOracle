//! Variable Selection Module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Method for selecting core variables
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SelectionMethod {
    /// Mutual information-based selection
    MutualInformation,
    /// Variance-based selection (select high-variance variables)
    Variance,
    /// Correlation-based selection
    Correlation,
    /// Feature importance from historical data
    FeatureImportance,
    /// Combined score (weighted average of multiple methods)
    Combined,
}

/// Selection score for variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionScore {
    /// Variable scores (higher = more important)
    pub scores: Vec<f64>,
    
    /// Selection method used
    pub method: SelectionMethod,
    
    /// Number of variables selected
    pub num_selected: usize,
}

/// Variable selector
pub struct VariableSelector {
    method: SelectionMethod,
}

impl VariableSelector {
    /// Create a new selector with specified method
    pub fn new(method: SelectionMethod) -> Self {
        Self { method }
    }
    
    /// Select variables based on intervention and response data
    pub fn select_variables(
        &self,
        intervention_data: &[f64],
        response_data: &[f64],
        context: Option<&HashMap<String, f64>>,
        max_variables: usize,
        min_variables: usize,
    ) -> Result<SelectionScore, String> {
        if intervention_data.is_empty() || response_data.is_empty() {
            return Err("Intervention or response data is empty".to_string());
        }
        
        let num_variables = intervention_data.len().min(response_data.len());
        
        let scores = match self.method {
            SelectionMethod::MutualInformation => {
                self.compute_mutual_information_scores(intervention_data, response_data)?
            }
            SelectionMethod::Variance => {
                self.compute_variance_scores(intervention_data, response_data)?
            }
            SelectionMethod::Correlation => {
                self.compute_correlation_scores(intervention_data, response_data)?
            }
            SelectionMethod::FeatureImportance => {
                self.compute_feature_importance_scores(intervention_data, response_data, context)?
            }
            SelectionMethod::Combined => {
                self.compute_combined_scores(intervention_data, response_data, context)?
            }
        };
        
        // Select top variables
        let num_selected = num_variables.clamp(min_variables, max_variables);
        
        // Normalize scores to 0-1 range
        let normalized_scores = self.normalize_scores(&scores);
        
        Ok(SelectionScore {
            scores: normalized_scores,
            method: self.method,
            num_selected,
        })
    }
    
    /// Compute mutual information scores
    fn compute_mutual_information_scores(
        &self,
        intervention_data: &[f64],
        response_data: &[f64],
    ) -> Result<Vec<f64>, String> {
        let mut scores = Vec::new();
        
        for i in 0..intervention_data.len().min(response_data.len()) {
            let mi = self.estimate_mutual_information(intervention_data[i], response_data[i]);
            scores.push(mi);
        }
        
        Ok(scores)
    }
    
    /// Estimate mutual information between two variables
    fn estimate_mutual_information(&self, x: f64, y: f64) -> f64 {
        // Simplified MI estimation: use correlation as proxy
        let correlation = (x * y).signum();
        let magnitude = (x.abs() * y.abs()).sqrt();
        
        // MI >= 0, and increases with absolute correlation
        correlation.abs() * magnitude.ln_1p()
    }
    
    /// Compute variance-based scores
    fn compute_variance_scores(
        &self,
        intervention_data: &[f64],
        response_data: &[f64],
    ) -> Result<Vec<f64>, String> {
        let mut scores = Vec::new();
        
        for i in 0..intervention_data.len().min(response_data.len()) {
            // Higher variance in response -> more informative
            let variance = response_data[i].powi(2);
            scores.push(variance);
        }
        
        Ok(scores)
    }
    
    /// Compute correlation-based scores
    fn compute_correlation_scores(
        &self,
        intervention_data: &[f64],
        response_data: &[f64],
    ) -> Result<Vec<f64>, String> {
        let mut scores = Vec::new();
        
        for i in 0..intervention_data.len().min(response_data.len()) {
            let correlation = self.compute_correlation(intervention_data[i], response_data[i]);
            scores.push(correlation.abs());
        }
        
        Ok(scores)
    }
    
    /// Compute correlation between intervention and response
    fn compute_correlation(&self, x: f64, y: f64) -> f64 {
        if x == 0.0 || y == 0.0 {
            return 0.0;
        }
        
        // Normalize and compute correlation
        let sign = (x * y).signum();
        let magnitude = x.abs().min(y.abs()) / x.abs().max(y.abs());
        sign * magnitude
    }
    
    /// Compute feature importance scores from context
    fn compute_feature_importance_scores(
        &self,
        intervention_data: &[f64],
        response_data: &[f64],
        context: Option<&HashMap<String, f64>>,
    ) -> Result<Vec<f64>, String> {
        let mut scores = Vec::new();
        
        for i in 0..intervention_data.len().min(response_data.len()) {
            // Base score from intervention-response relationship
            let base_score = (intervention_data[i].abs() + response_data[i].abs()) / 2.0;
            
            // Adjust by context if available
            let context_adjustment = if let Some(ctx) = context {
                ctx.get(&format!("var_{}", i))
                    .copied()
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            
            scores.push(base_score + context_adjustment);
        }
        
        Ok(scores)
    }
    
    /// Compute combined scores from multiple methods
    fn compute_combined_scores(
        &self,
        intervention_data: &[f64],
        response_data: &[f64],
        context: Option<&HashMap<String, f64>>,
    ) -> Result<Vec<f64>, String> {
        // Compute scores from each method
        let mi_scores = self.compute_mutual_information_scores(intervention_data, response_data)?;
        let var_scores = self.compute_variance_scores(intervention_data, response_data)?;
        let corr_scores = self.compute_correlation_scores(intervention_data, response_data)?;
        let fi_scores = self.compute_feature_importance_scores(intervention_data, response_data, context)?;
        
        // Combine with weights
        let mut combined_scores = Vec::new();
        for i in 0..mi_scores.len() {
            let combined = 0.3 * mi_scores[i]
                + 0.2 * var_scores[i]
                + 0.3 * corr_scores[i]
                + 0.2 * fi_scores[i];
            combined_scores.push(combined);
        }
        
        Ok(combined_scores)
    }
    
    /// Normalize scores to 0-1 range
    fn normalize_scores(&self, scores: &[f64]) -> Vec<f64> {
        if scores.is_empty() {
            return Vec::new();
        }
        
        let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_score = scores.iter().cloned().fold(f64::INFINITY, f64::min);
        
        if max_score == min_score {
            return scores.iter().map(|_| 0.5).collect();
        }
        
        scores.iter()
            .map(|&s| (s - min_score) / (max_score - min_score))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_selector_creation() {
        let selector = VariableSelector::new(SelectionMethod::MutualInformation);
        assert_eq!(selector.method, SelectionMethod::MutualInformation);
    }
    
    #[test]
    fn test_select_variables() {
        let selector = VariableSelector::new(SelectionMethod::Variance);
        
        let intervention = vec![1.0, 0.5, 2.0, 0.3];
        let response = vec![0.8, 0.4, 1.6, 0.2];
        
        let result = selector.select_variables(&intervention, &response, None, 5, 3);
        
        assert!(result.is_ok());
        let scores = result.unwrap();
        assert_eq!(scores.num_selected, 4); // Clamp to actual data size
        assert!(!scores.scores.is_empty());
    }
    
    #[test]
    fn test_normalize_scores() {
        let selector = VariableSelector::new(SelectionMethod::Variance);
        
        let scores = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = selector.normalize_scores(&scores);
        
        assert_eq!(normalized[0], 0.0);
        assert_eq!(normalized[4], 1.0);
        assert!(normalized[2] > 0.4 && normalized[2] < 0.6);
    }
    
    #[test]
    fn test_empty_data() {
        let selector = VariableSelector::new(SelectionMethod::MutualInformation);
        
        let result = selector.select_variables(&[], &[], None, 5, 3);
        
        assert!(result.is_err());
    }
}
