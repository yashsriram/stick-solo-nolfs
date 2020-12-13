extern crate stick_solo;
use bevy::prelude::*;
use ndarray::prelude::*;
use stick_solo::act::switchable_nr_couple::SwitchableNRCouple;
use stick_solo::game::{
    base_plugins::BasePlugins,
    camera_plugin::CameraPlugin,
    goal_couple_plugin::{GoalCouple, GoalCouplePlugin},
    pause_plugin::Pause,
    pause_plugin::PausePlugin,
    status_bar_plugin::{StatusBarPlugin, Ticks},
    switchable_nr_couple_plugin::SwitchableNRCouplePlugin,
};
use stick_solo::plan::gradient_descent::*;

fn main() {
    let inf = f32::INFINITY;
    let pi = std::f32::consts::PI;
    App::build()
        .add_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_resource(WindowDescriptor {
            width: 2000,
            height: 1000,
            ..Default::default()
        })
        .add_plugin(GoalCouplePlugin::new(GoalCouple(
            Vec2::new(0.2, -0.2),
            Vec2::new(0.5, -0.0),
        )))
        .add_plugins(BasePlugins)
        .add_plugin(CameraPlugin)
        .add_plugin(SwitchableNRCouplePlugin::new(
            SwitchableNRCouple::new_left_pivot(
                Vec2::new(0.0, -0.1),
                &[0.3, 0.2],
                &[0.1, 0.1],
                &[(-inf, inf), (0.0, pi)],
                &[0.2, 0.3],
                &[0.1, 0.2],
                &[(-inf, inf), (0.0, pi)],
                0.01,
            ),
        ))
        .add_plugin(StatusBarPlugin)
        .add_plugin(PausePlugin)
        .add_system(control.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}

fn control(
    mut agent: ResMut<SwitchableNRCouple>,
    pause: Res<Pause>,
    mut ticks: ResMut<Ticks>,
    goal_couple: ResMut<GoalCouple>,
) {
    // Pause => pause everything
    if pause.0 {
        return;
    }
    {
        let (_, origin, ls, qs, _, _) = agent.left().get_current_state();

        let given_goal = goal_couple.0;
        let (take_end_to_given_goal, push_com_x_from_its_goal, push_com_y_upward) =
            gradient_descent(
                origin,
                ls,
                qs,
                &given_goal,
                EndControl::JacobianTranspose,
                COMXGoalType::PivotGoalMidpoint,
            );

        let beta = 1.0;
        let gamma = 0.1;
        let delta = 0.1 / (1.0 + ticks.0 as f32).powf(1.0);
        agent.update(
            beta * take_end_to_given_goal
                + gamma * -push_com_x_from_its_goal
                + delta * -push_com_y_upward,
            arr1(&[0.0, 0.0]),
        );
    }
    {
        let (_, origin, ls, qs, _, _) = agent.right().get_current_state();

        let given_goal = goal_couple.1;
        let (take_end_to_given_goal, push_com_x_from_its_goal, push_com_y_upward) =
            gradient_descent(
                origin,
                ls,
                qs,
                &given_goal,
                EndControl::JacobianTranspose,
                COMXGoalType::PivotGoalMidpoint,
            );

        let beta = 1.0;
        let gamma = 0.1;
        let delta = 0.1 / (1.0 + ticks.0 as f32).powf(1.0);
        agent.update(
            arr1(&[0.0, 0.0]),
            beta * take_end_to_given_goal
                + gamma * -push_com_x_from_its_goal
                + delta * -push_com_y_upward,
        );
    }
    ticks.0 += 1;
}
