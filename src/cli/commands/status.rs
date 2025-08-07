use crate::terminal::output::{GLOBAL_TASK_MONITOR, info_text, success_text, error_text};
use std::io::{self, Write};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", info_text("📊 Checking system status..."));
    
    // Display the status dashboard
    let dashboard = GLOBAL_TASK_MONITOR.render_status_dashboard(80);
    println!("{}", dashboard);
    
    // Show unread notifications count
    let unread_notifications = GLOBAL_TASK_MONITOR.get_unread_notifications();
    if !unread_notifications.is_empty() {
        println!("{} {} unread notifications", 
            info_text("📬"), 
            unread_notifications.len());
        
        println!("Run 'forge status --clear' to mark all as read");
    } else {
        println!("{}", success_text("✅ No pending notifications"));
    }
    
    Ok(())
}

pub fn run_with_args(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return run();
    }
    
    match args[0].as_str() {
        "--clear" => {
            let notifications = GLOBAL_TASK_MONITOR.get_unread_notifications();
            for notification in notifications {
                GLOBAL_TASK_MONITOR.mark_notification_read(&notification.id);
            }
            println!("{}", success_text("✅ All notifications marked as read"));
            Ok(())
        }
        "--demo" => {
            demo_progress_system()
        }
        _ => {
            eprintln!("{}", error_text("❌ Unknown status command option"));
            eprintln!("Available options: --clear, --demo");
            Ok(())
        }
    }
}

fn demo_progress_system() -> Result<(), Box<dyn std::error::Error>> {
    use crate::terminal::output::{MultiStageProgress, TerminalControl};
    use std::thread;
    use std::time::Duration;
    
    println!("{}", info_text("🎬 Demonstrating enhanced progress system..."));
    println!("{}", TerminalControl::hide_cursor());
    
    // Create a demo task in the background monitor
    let task_id = GLOBAL_TASK_MONITOR.start_task("demo", "Demo Multi-Stage Process");
    
    // Create a multi-stage progress indicator
    let mut progress = MultiStageProgress::new("Demo Multi-Stage Process");
    progress.add_stage("Initialize", 0.2);
    progress.add_stage("Processing Data", 0.5);
    progress.add_stage("Validation", 0.2);
    progress.add_stage("Cleanup", 0.1);
    
    // Update the task with our progress structure
    GLOBAL_TASK_MONITOR.update_task(&task_id, |task| {
        task.progress = progress.clone();
    })?;
    
    // Simulate the stages
    for stage_idx in 0..4 {
        progress.start_stage(stage_idx)?;
        
        // Update task in monitor
        GLOBAL_TASK_MONITOR.update_task(&task_id, |task| {
            task.progress = progress.clone();
        })?;
        
        let _stage_name = match stage_idx {
            0 => "Initialize",
            1 => "Processing Data", 
            2 => "Validation",
            3 => "Cleanup",
            _ => "Unknown"
        };
        
        // Simulate progress within this stage
        for i in 0..=10 {
            let stage_progress = i as f64 / 10.0;
            let sub_message = match stage_idx {
                0 => Some(format!("Setting up environment... {}/10", i)),
                1 => Some(format!("Processing file {} of 10", i)),
                2 => Some(format!("Validating rule {} of 10", i)),
                3 => Some(format!("Cleaning up temp files... {}/10", i)),
                _ => None,
            };
            
            progress.update_stage_progress(stage_idx, stage_progress, sub_message.as_deref())?;
            
            // Update task in monitor
            GLOBAL_TASK_MONITOR.update_task(&task_id, |task| {
                task.progress = progress.clone();
            })?;
            
            // Clear screen and render current progress
            print!("{}{}", TerminalControl::clear_screen(), progress.render(50));
            
            // Show ETA if available
            if let Some(eta) = progress.get_eta_seconds() {
                println!("\n⏰ ETA: {}s remaining", eta);
            }
            
            io::stdout().flush()?;
            thread::sleep(Duration::from_millis(300));
        }
        
        progress.complete_stage(stage_idx)?;
    }
    
    // Complete the task
    GLOBAL_TASK_MONITOR.complete_task(&task_id, "Demo completed successfully!");
    
    // Final display
    print!("{}{}", TerminalControl::clear_screen(), progress.render(50));
    println!("{}", TerminalControl::show_cursor());
    println!("\n{}", success_text("✅ Demo completed! Check notifications with 'forge status'"));
    
    Ok(())
}