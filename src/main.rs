use clap::Parser;
use serde::{Deserialize, Serialize};
use reqwest;
use chrono::{NaiveDate, Local, Datelike};
use std::fs;
use serde_json;

#[derive(Parser, Debug)]
struct Args { 
    /// Country Code
    country: String,
}

#[derive(Deserialize, Serialize,  Debug, Clone)]
struct Holiday { 
    date: String,
    name: String,
    counties: Option<Vec<String>>, // Counties information is optional
    types: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CachedData { 
    country_code: String,
    date: String, 
    holidays: Vec<Holiday>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FullCache {
    date: String,             
    data: Vec<CachedData>,    
}

const CACHE_FILE: &str = "holidays_cache.json" ; // cache file where data will be saved
const COUNTRY_CODES_FILE: &str = "country_codes.txt"; // Name of the file containing country codes

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let args = Args::parse();
    let country_code = args.country.to_uppercase();
    let valid_country_codes = read_country_codes().expect("Failed to read country codes file");


    if !valid_country_codes.contains(&country_code) {
        eprintln!(
            "Error: '{}' is not a valid country code. Valid country codes are: {:?}",
            country_code, valid_country_codes
        );
        std::process::exit(1);
    }
    
    let today = Local::now().date_naive(); 
    let current_year = Local::now().year();

    reset_cache_if_needed(today)?; //  If the date of the cache file and today's date are different, it clears the file.

    if let Some(cached_data) = check_cache(&country_code, today)? {
        println!("Using cached data for {} (Date: {}).", country_code, today);

        print_holidays(&cached_data.holidays, today);
                return Ok(()); // Cache was used
            }

    let url = format!("https://date.nager.at/api/v3/publicholidays/{}/{}", current_year, country_code); 

    // Request to API
    match reqwest::get(&url).await {
        Ok(response) => {
            if response.status().is_success() {
                let holidays: Vec<Holiday> = response.json().await?;
                write_cache(&country_code, today, &holidays)?;
                print_holidays(&holidays, today);
            } else {
                handle_http_error(response.status());
            }
        }
        Err(err) => {
            if err.is_connect() {
                eprintln!("Network error: Unable to connect to the API. Please check your internet connection.");
            } else if err.is_timeout() {
                eprintln!("Request timed out: Please try again later.");
            } else {
                eprintln!("Unexpected error occurred while connecting to the API: {}", err);
            }
            std::process::exit(1);
        }
    }
    
    Ok(())
}

fn read_country_codes() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    match fs::read_to_string(COUNTRY_CODES_FILE) {
        Ok(content) => Ok(content.lines().map(|line| line.trim().to_string()).collect()),
        Err(err) => {
            handle_file_error(&err, COUNTRY_CODES_FILE);
            Err(Box::new(err)) 
        }
    }
}

fn check_cache(country_code: &str, today: NaiveDate) -> Result<Option<CachedData>, Box<dyn std::error::Error>> {
    if let Ok(cache_content) = fs::read_to_string(CACHE_FILE) {
        if let Ok(full_cache) = serde_json::from_str::<FullCache>(&cache_content) {
            if let Some(cached_data) = full_cache.data.iter().find(|data| {
                data.country_code == country_code && data.date == today.to_string()
            }) {
                return Ok(Some(cached_data.clone()));
            }
        } else {
            eprintln!("Warning: Cache file exists but could not be parsed. Ignoring cache.");
        }
    } else {
        eprintln!("Warning: Cache file could not be opened or does not exist. Proceeding with API request.");
    }

    Ok(None) 
}

fn print_holidays(holidays: &[Holiday], today: NaiveDate) {
    let filtered_holidays: Vec<&Holiday> = holidays
        .iter()
        .filter(|holiday| {
            NaiveDate::parse_from_str(&holiday.date, "%Y-%m-%d")
                .map(|date| date > today) 
                .unwrap_or(false)
        })
        .take(5) // first 5 holiday
        .collect();

    for holiday in filtered_holidays {
        println!(
            "Date: {}, Name: {}, Counties: {}, Types: {}",
            holiday.date,
            holiday.name,
            match &holiday.counties {
                Some(counties) => counties.join(", "),
                None => "National".to_string(),
            },
            if holiday.types.len() == 1 {
                holiday.types[0].clone()
            } else {
                holiday.types.join(", ")
            }
        );
    }
}

fn write_cache(country_code: &str, today: NaiveDate, holidays: &[Holiday],) -> Result<(), Box<dyn std::error::Error>> {
    // read current cache
    let mut full_cache: FullCache = if let Ok(cache_content) = fs::read_to_string(CACHE_FILE) {
        serde_json::from_str(&cache_content).unwrap_or_else(|_| FullCache {
            date: today.to_string(),
            data: Vec::new(),
        })
    } else {
        FullCache {
            date: today.to_string(),
            data: Vec::new(),
        }
    };

    // cache check for same day and country code
    if full_cache.data.iter().any(|data| {
        data.country_code == country_code && data.date == today.to_string()
    }) {
        println!("Cache already contains data for {} on {}.", country_code, today);
        return Ok(());
    }

    // create new cache data
    let new_cached_data = CachedData {
        country_code: country_code.to_string(),
        date: today.to_string(),
        holidays: holidays.to_vec(),
    };

    // adding new data without deleting old data
    full_cache.data.push(new_cached_data);

    // Update cache file
    let cache_content = serde_json::to_string(&full_cache)?;
     fs::write(CACHE_FILE, cache_content).map_err(|err| {
            handle_file_error(&err, CACHE_FILE);
            err
        })?;
    println!("Cache updated successfully for {}.", country_code);

    Ok(())
}

fn reset_cache_if_needed(today: NaiveDate) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(cache_content) = fs::read_to_string(CACHE_FILE) {
        if let Ok(full_cache) = serde_json::from_str::<FullCache>(&cache_content) {
            // Check cache date
            if full_cache.date != today.to_string() {
                println!("New day detected. Resetting cache...");
                // It is a new day so clean cache
                let new_cache = FullCache {
                    date: today.to_string(),
                    data: Vec::new(),
                };
                let cache_content = serde_json::to_string(&new_cache)?;
                fs::write(CACHE_FILE, cache_content).map_err(|err| {
                    handle_file_error(&err, CACHE_FILE);
                    err
                })?;
            }
        }
    } else {
        // If there is no cache file, create a new one
        let new_cache = FullCache {
            date: today.to_string(),
            data: Vec::new(),
        };
        let cache_content = serde_json::to_string(&new_cache)?;
           fs::write(CACHE_FILE, cache_content).map_err(|err| {
            handle_file_error(&err, CACHE_FILE);
            err
        })?;
    }

    Ok(())
}

fn handle_http_error(status: reqwest::StatusCode) {
    match status.as_u16() {
        400 => {
            eprintln!("Error: Bad Request.");
        }
        404 => {
            eprintln!("Error: Not Found.");
        }
        500 => {
            eprintln!("Error: Internal Server Error.");
        }
        503 => {
            eprintln!("Error: Service Unavailable.");
        }
        _ => {
            eprintln!("Error: Unexpected HTTP status: {}", status);
        }
    }
    std::process::exit(1);
}

fn handle_file_error(err: &std::io::Error, file_name: &str) {
    match err.kind() {
        std::io::ErrorKind::NotFound => {
            eprintln!("Error: The file '{}' was not found.", file_name);
        }
        std::io::ErrorKind::PermissionDenied => {
            eprintln!("Error: Permission denied while accessing '{}'.", file_name);
        }
        _ => {
            eprintln!("Error: An unexpected error occurred with '{}': {}", file_name, err);
        }
    }
    std::process::exit(1);
}

