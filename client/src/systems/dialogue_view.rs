use bevy::prelude::*;
use bevy_yarnspinner::prelude::*;
use bevy_yarnspinner::events::{PresentLineEvent, PresentOptionsEvent, DialogueCompleteEvent};
use bevy_yarnspinner::prelude::OptionId;

pub struct SimpleDialogueViewPlugin;

impl Plugin for SimpleDialogueViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            debug_all_dialogue_events,
            handle_present_line_events,
            handle_present_options_events,
            handle_dialogue_complete_events,
        ));
    }
}

/// System to debug all dialogue-related events and state
fn debug_all_dialogue_events(
    mut line_events: EventReader<PresentLineEvent>,
    mut option_events: EventReader<PresentOptionsEvent>,
    mut complete_events: EventReader<DialogueCompleteEvent>,
    dialogue_runners: Query<&DialogueRunner>,
    mut frame_counter: Local<u32>,
) {
    *frame_counter += 1;
    
    let _line_count = line_events.len();
    let _option_count = option_events.len();
    let _complete_count = complete_events.len();
    let _runner_count = dialogue_runners.iter().count();
    
    // Always log if we have any events (these are critical!)
    // if line_count > 0 || option_count > 0 || complete_count > 0 {
    //     warn!("🚨 DIALOGUE EVENTS DETECTED ON FRAME {}! Lines: {}, Options: {}, Complete: {}, Runners: {}", 
    //           *frame_counter, line_count, option_count, complete_count, runner_count);
    // }
    
    // Log individual events for detailed debugging
    for event in line_events.read() {
        info!("💬 DIALOGUE: {}", event.line.text);
        // warn!("  📍 Line ID: {:?}", event.line.id);
    }
    
    for event in option_events.read() {
        info!("🔸 DIALOGUE CHOICES:");
        for (i, option) in event.options.iter().enumerate() {
            info!("  [{}] {}", i + 1, option.line.text);
        }
        // warn!("🎯 OPTIONS EVENT: {} choices available", event.options.len());
        // for (i, option) in event.options.iter().enumerate() {
        //     warn!("  [{}] '{}' (ID: {:?})", i, option.line.text, option.line.id);
        // }
    }
    
    for _event in complete_events.read() {
        info!("✅ DIALOGUE COMPLETE - Book interaction finished!");
        // warn!("✅ DIALOGUE COMPLETE EVENT on frame {}", *frame_counter);
    }
    
    // Every 60 frames (roughly 1 second), log runner status
    // if *frame_counter % 60 == 0 && runner_count > 0 {
    //     for (i, runner) in dialogue_runners.iter().enumerate() {
    //         info!("🎭 Runner #{}: running={}, current_node={:?}", 
    //               i, runner.is_running(), runner.current_node());
    //     }
    // }
}

/// System to handle line presentation events
fn handle_present_line_events(
    mut line_events: EventReader<PresentLineEvent>,
    mut dialogue_runners: Query<&mut DialogueRunner>,
) {
    for event in line_events.read() {
        // Display the dialogue line to the user
        info!("💬 DIALOGUE: {}", event.line.text);
        
        // CRITICAL: Must call continue_in_next_update() to progress dialogue
        if let Ok(mut dialogue_runner) = dialogue_runners.single_mut() {
            dialogue_runner.continue_in_next_update();
            // info!("✅ Called continue_in_next_update() - dialogue should progress");
        } else {
            // warn!("❌ No DialogueRunner found when trying to continue dialogue");
        }
    }
}

/// System to handle choice presentation events
fn handle_present_options_events(
    mut option_events: EventReader<PresentOptionsEvent>,
    mut dialogue_runners: Query<&mut DialogueRunner>,
) {
    for event in option_events.read() {
        info!("🔸 DIALOGUE CHOICES:");
        for (index, option) in event.options.iter().enumerate() {
            info!("  [{}] {}", index + 1, option.line.text);
        }
        
        // For now, automatically choose the first option
        // In a real game, you'd wait for user input to select
        if !event.options.is_empty() {
            info!("🎯 Auto-selecting first choice...");
            if let Ok(mut dialogue_runner) = dialogue_runners.single_mut() {
                match dialogue_runner.select_option(OptionId(0)) {
                    Ok(_) => {
                        // info!("✅ Successfully selected option 0");
                        // Continue dialogue after selection
                        dialogue_runner.continue_in_next_update();
                    }
                    Err(_e) => {
                        // warn!("❌ Failed to select option 0: {:?}", e);
                    }
                }
            } else {
                // warn!("❌ No DialogueRunner found when trying to select option");
            }
        }
    }
}

/// System to handle dialogue completion
fn handle_dialogue_complete_events(
    mut complete_events: EventReader<DialogueCompleteEvent>,
) {
    for _event in complete_events.read() {
        info!("✅ DIALOGUE COMPLETE - Book interaction finished!");
    }
}