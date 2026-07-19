use chrono::{Local, Utc};
use cron_tab::Cron;

fn scheduler() {
// Create a new cron scheduler with UTC timezone
let mut cron = Cron::new(Utc);

// Add a job that runs every 10 seconds
let job_id = cron.add_fn("*/10 * * * * * *", || {
println!("Job executed at: {}", Local::now());
}).expect("Failed to add job");

// Start the scheduler in background
cron.start();

// Add another job that runs every minute
cron.add_fn("0 * * * * * *", || {
println!("Every minute job executed!");
}).expect("Failed to add job");

// Remove the first job after some time
std::thread::sleep(std::time::Duration::from_secs(1));
cron.remove(job_id);

// Stop the scheduler
cron.stop();
}