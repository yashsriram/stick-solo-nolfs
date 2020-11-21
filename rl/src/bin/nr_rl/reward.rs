use super::ceo::Reward;
use super::encode::generate_input;
use super::fcn::*;
use bevy::prelude::*;
use ndarray::prelude::*;
use serde::{Deserialize, Serialize};
use stick_solo::act::NRAgent;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NRAgentReward {
    pub origin: Vec2,
    pub ls: Vec<f32>,
    pub qs: Vec<f32>,
    pub goal: Vec2,
}

impl Reward for NRAgentReward {
    fn at(
        &self,
        fcn: &FCN,
        params: &Array1<f32>,
        num_episodes: usize,
        num_episode_ticks: usize,
    ) -> f32 {
        let mut cumulative_reward = 0.0;
        for _ in 0..num_episodes {
            // Spawn agent
            let mut agent = NRAgent::new(self.origin, &self.ls, &self.qs, 1.0);
            // Start calculating reward
            let mut episode_reward = 0.0;
            for _tick in 0..num_episode_ticks {
                let input = generate_input(agent.get_current_state(), &self.goal);
                let delta_qs = fcn.at_with(&input, params);
                let delta_qs_norm = delta_qs.mapv(|e| e * e).sum().sqrt();
                // let prev_delta_qs = agent.get_current_control();
                // let delta_delta_qs_norm = (delta_qs.clone() - prev_delta_qs)
                //     .mapv(|e| e * e)
                //     .sum()
                //     .sqrt();
                // Apply control
                agent.update(delta_qs);
                // Makes agent translate towards goal
                let last_vertex = agent.get_last_vertex();
                let dist = (last_vertex - self.goal).length();
                episode_reward -= dist * 100.0;
                // Penalize huge controls
                episode_reward -= delta_qs_norm;
                // Penalize huge difference in controls
                // episode_reward -= delta_delta_qs_norm;
            }
            // Makes agent reach the goal at the end of episode
            let last_vertex = agent.get_last_vertex();
            let final_dist = (last_vertex - self.goal).length();
            episode_reward += 1000.0 * (-final_dist).exp();
            // Makes agent stop at the end of episode
            let delta_qs = agent.get_current_control();
            let delta_qs_norm = delta_qs.mapv(|e| e * e).sum().sqrt();
            episode_reward += 10000.0 * (-delta_qs_norm).exp() * (-final_dist).exp();

            cumulative_reward += episode_reward;
        }

        let average_reward = cumulative_reward / num_episodes as f32;
        average_reward
    }
}
