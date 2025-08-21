pub mod color;
pub use color::{convert_to_color, is_valid_color, shorten_color};

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static VALIDATION_TIME: AtomicU64 = AtomicU64::new(0);
static CONVERSION_TIME: AtomicU64 = AtomicU64::new(0);
static SHORTENING_TIME: AtomicU64 = AtomicU64::new(0);
static TOTAL_TIME: AtomicU64 = AtomicU64::new(0);
static EARLY_RETURN_TIME: AtomicU64 = AtomicU64::new(0);
static MEASUREMENT_COUNT: AtomicU64 = AtomicU64::new(0);
static EARLY_RETURN_COUNT: AtomicU64 = AtomicU64::new(0);

fn duration_to_nanos(duration: Duration) -> u64 {
    duration.as_nanos() as u64
}

fn nanos_to_duration(nanos: u64) -> Duration {
    Duration::from_nanos(nanos)
}

#[inline(always)]
fn add_time(atomic: &AtomicU64, duration: Duration) {
    atomic.fetch_add(duration_to_nanos(duration), Ordering::Relaxed);
}

pub fn shorten_css_color(color_str: &str) -> String {
    let start = Instant::now();
    
    let trimmed = color_str.trim();
    
    // Early return для уже оптимальных случаев
    if let Some(result) = check_early_return_cases(trimmed) {
        let early_return_time = start.elapsed();
        add_time(&EARLY_RETURN_TIME, early_return_time);
        add_time(&TOTAL_TIME, early_return_time);
        EARLY_RETURN_COUNT.fetch_add(1, Ordering::Relaxed);
        MEASUREMENT_COUNT.fetch_add(1, Ordering::Relaxed);
        return result;
    }

    let validation_start = Instant::now();
    let is_valid = is_valid_color(color_str);
    let validation_time = validation_start.elapsed();

    if !is_valid {
        let total_time = start.elapsed();
        add_time(&VALIDATION_TIME, validation_time);
        add_time(&TOTAL_TIME, total_time);
        MEASUREMENT_COUNT.fetch_add(1, Ordering::Relaxed);
        return color_str.to_string();
    }

    let conversion_start = Instant::now();
    if let Some(color) = convert_to_color(color_str) {
        let conversion_time = conversion_start.elapsed();

        let shortening_start = Instant::now();
        let shortened = shorten_color(&color);
        let shortening_time = shortening_start.elapsed();

        let total_time = start.elapsed();

        add_time(&VALIDATION_TIME, validation_time);
        add_time(&CONVERSION_TIME, conversion_time);
        add_time(&SHORTENING_TIME, shortening_time);
        add_time(&TOTAL_TIME, total_time);
        MEASUREMENT_COUNT.fetch_add(1, Ordering::Relaxed);

        shortened
    } else {
        let conversion_time = conversion_start.elapsed();
        let total_time = start.elapsed();

        add_time(&VALIDATION_TIME, validation_time);
        add_time(&CONVERSION_TIME, conversion_time);
        add_time(&TOTAL_TIME, total_time);
        MEASUREMENT_COUNT.fetch_add(1, Ordering::Relaxed);

        color_str.to_string()
    }
}

#[inline(always)]
fn check_early_return_cases(trimmed: &str) -> Option<String> {
    let len = trimmed.len();
    
    // Короткие hex коды (уже оптимальны)
    if (len == 4 || len == 5) && trimmed.starts_with('#') {
        return Some(trimmed.to_string());
    }
    
    // Короткие именованные цвета (невозможно оптимизировать дальше)
    if len <= 4 {
        match trimmed.to_ascii_lowercase().as_str() {
            "red" | "blue" | "lime" | "aqua" | "cyan" | "gold" | "gray" | "grey" | "navy" | "peru" | "pink" | "plum" | "snow" | "tan" => {
                return Some(trimmed.to_lowercase());
            }
            _ => {}
        }
    }
    
    None
}

pub fn print_benchmark_summary() {
    let count = MEASUREMENT_COUNT.load(Ordering::Relaxed);
    let early_returns = EARLY_RETURN_COUNT.load(Ordering::Relaxed);

    if count == 0 {
        println!("No measurements recorded");
        return;
    }

    let processed_count = count - early_returns;
    
    let avg_validation = if processed_count > 0 {
        nanos_to_duration(VALIDATION_TIME.load(Ordering::Relaxed) / processed_count)
    } else {
        Duration::from_nanos(0)
    };
    
    let avg_conversion = if processed_count > 0 {
        nanos_to_duration(CONVERSION_TIME.load(Ordering::Relaxed) / processed_count)
    } else {
        Duration::from_nanos(0)
    };
    
    let avg_shortening = if processed_count > 0 {
        nanos_to_duration(SHORTENING_TIME.load(Ordering::Relaxed) / processed_count)
    } else {
        Duration::from_nanos(0)
    };
    
    let avg_total = nanos_to_duration(TOTAL_TIME.load(Ordering::Relaxed) / count);
    let avg_early_return = if early_returns > 0 {
        nanos_to_duration(EARLY_RETURN_TIME.load(Ordering::Relaxed) / early_returns)
    } else {
        Duration::from_nanos(0)
    };

    println!("\n📊 СРЕДНЯЯ ПРОИЗВОДИТЕЛЬНОСТЬ:");
    println!("   Валидация:     {:?}", avg_validation);
    println!("   Конверсия:     {:?}", avg_conversion);
    println!("   Сокращение:    {:?}", avg_shortening);
    println!("   Early Return:  {:?}", avg_early_return);
    println!("   Общее время:   {:?}", avg_total);
    println!("   Измерений:     {}", count);
    println!("   Early Returns: {} ({:.1}%)", early_returns, 
             early_returns as f64 / count as f64 * 100.0);
}

pub fn reset_benchmark_stats() {
    VALIDATION_TIME.store(0, Ordering::Relaxed);
    CONVERSION_TIME.store(0, Ordering::Relaxed);
    SHORTENING_TIME.store(0, Ordering::Relaxed);
    TOTAL_TIME.store(0, Ordering::Relaxed);
    EARLY_RETURN_TIME.store(0, Ordering::Relaxed);
    MEASUREMENT_COUNT.store(0, Ordering::Relaxed);
    EARLY_RETURN_COUNT.store(0, Ordering::Relaxed);
}