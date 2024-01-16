use std::cmp::max;
use std::fs;
use std::time::Duration;
use thirtyfour::prelude::*;
use clap::{Parser, Subcommand};
use lettre::{Message, SmtpTransport, Transport, transport::smtp};
use lettre::message::{Body, header};


#[derive(Parser)]
#[command(author, version, about)]
struct CLi {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    /// scrapes the specified URL
    Scrape {
        /// url to scrape from
        // #[arg(long)]
        url: String,

        /// location to save scraped data to
        #[arg(short, long, default_value= "./contact_data.csv")]
        output_path: String,
    },

    /// sends emails to the specified addresses
    Apply {
        /// .md file with message contents
        message: String,

        /// location of input file
        #[arg(short, long)]
        input_path: String,

        /// SMTP server URL
        #[arg(long)]
        url: String,

        /// SMTP user
        #[arg(short, long)]
        user: String,

        /// SMTP password
        #[arg(short, long)]
        pass: String,
    }
}


#[tokio::main]
async fn main() {
    let args = CLi::parse();

    match &args.command {
        Commands::Scrape {url, output_path} => {
            scrape_emails(url, output_path).await.expect("Failed to scrape emails");
        },
        Commands::Apply { message, input_path, url, user, pass} => {
            stagger_emails(input_path, url, user, pass, message).await.expect("Failed to send emails");
        }
    }
    // scrape_emails().await.expect("Failed to scrape emails");


}

async fn scrape_emails(url: &String, out_path: &String) -> WebDriverResult<()> {
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", caps).await.expect("Failed to create WebDriver. Did you start ChromeDriver?");

    let mut wtr = csv::Writer::from_path(out_path).expect("unable to write file");

    println!("Navigating to website");
    driver.goto(url).await?;
    //fullscreen window to ensure consistent element placement
    driver.fullscreen_window().await?;

    //filter out everything except KoSi
    println!("Finding and clicking not KoSi subjects");
    let kits = driver.find(By::XPath("/html/body/div/div[2]/div/div/div/div[1]/div[2]/div[1]/div[2]")).await?;
    let master_dual = driver.find(By::XPath("/html/body/div/div[2]/div/div/div/div[1]/div[2]/div[2]/div[1]")).await?;
    let data_science = driver.find(By::XPath("/html/body/div/div[2]/div/div/div/div[1]/div[2]/div[2]/div[2]")).await?;
    kits.click().await?;
    master_dual.click().await?;
    data_science.click().await?;

    //find list view toggle
    println!("Finding and clicking list view toggle");
    let view_toggle = driver.find(By::XPath("/html/body/div/div[2]/div/div/div/div[2]")).await?;
    //switch to list view
    view_toggle.click().await?;

    //give website time to update
    println!("Waiting 3 seconds...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    println!("Finding contact elements");
    let contact_elems = driver.find_all(By::Css(".w-full.items-center.flex.flex-col")).await?;
    println!("Found {} potential contacts", contact_elems.len());

    let mut contact_data: Vec<(String, String, String)> = vec![];
    wtr.serialize(("Company", "Contact name", "Contact email")).expect("unable to serialize fields");

    for contact_elem in contact_elems {
        let company = contact_elem.find(By::Css("h2")).await?.text().await?;
        let contact_name = contact_elem.find(By::Css("p.w-full")).await?.text().await.unwrap_or("None".to_string());
        let email = contact_elem.find(By::Css(".underline")).await?.text().await.unwrap_or("None".to_string()).replace("(at)", "@");

        //skip duplicates and empty company names
        if company.is_empty() || company == contact_data.last().unwrap_or(&("".to_string(), "".to_string(), "".to_string())).0 { continue; }

        let contact = (company, contact_name, email);

        println!("{}: {} - {}", contact.0, contact.1, contact.2);

        contact_data.push(contact);
        wtr.serialize(contact_data.last().expect("failed to push contact to vec")).expect("unable to serialise data");
    }

    wtr.flush().expect("unable to flush data");

    driver.quit().await?;
    Ok(())
}

async fn stagger_emails(input_path: &String, smtp_url: &str, smtp_user: &str, smtp_pass: &str, message: &String) -> Result<(), csv::Error> {
    let mut targets: Vec<(String, String, String)> = vec![];
    let mut reader = csv::Reader::from_path(input_path).expect("Failed to open csv input");

    for result in reader.deserialize() {
        let record: (String, String, String) = result?;
        //don't push records with empty email fields
        //or empty name fields
        if !record.2.is_empty() && !record.1.is_empty() {
            targets.push(record);
        }
    }

    // exit if targets file contains no email addresses
    if targets.is_empty() {
        panic!("Targets file contains no email addresses")
    }

    let message = fs::read_to_string(message).expect("unable to read message file");

    const DELAY: Duration = Duration::from_secs(10);
    println!("Preparing to send E-Mails to {} targets.", targets.len());
    println!("Delay between emails: {:?} - ETA: {:?}", DELAY, DELAY* targets.len() as u32);
    let mut success_count = 0;
    for (i, target) in targets.iter().enumerate() {
        if i % 10 == 0 {
            print!("[{}-{}]", i, max(i+9, targets.len()));
        }

        match send_email(target, smtp_url, smtp_user, smtp_pass, &message).await {
            Ok(_) => {
                success_count += 1;
                print!(".");
            },
            Err(_) => print!("x")
        };

        //sleep after sending each email as not to trip rate limits
        tokio::time::sleep(DELAY).await;
    }

    println!();
    println!("Emails sent! {} successful, {} failed - {:.2}% success rate", success_count, targets.len()-success_count, (success_count/targets.len())*100);

    Ok(())
}

async fn send_email(target: &(String, String, String), smtp_url: &str, smtp_user: &str, smtp_pass: &str, message: &str) -> Result<smtp::response::Response, smtp::Error> {
    let email = Message::builder()
        .from(smtp_user.parse().expect("Failed to parse sender email"))
        .to(target.2.parse().expect("Failed to parse recipient email"))
        .header(header::ContentType::TEXT_HTML)
        .subject("Anfrage dualer Studienplatz")
        .body(Body::new(message.replace("{}", &target.1)))
        .unwrap();

    let creds = smtp::authentication::Credentials::new(smtp_user.parse().unwrap(), smtp_pass.parse().unwrap());

    let mailer = SmtpTransport::relay(smtp_url)
        .unwrap()
        .credentials(creds)
        .build();

    mailer.send(&email)
}