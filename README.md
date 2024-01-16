# scrape-apply
Scrape-apply is a small utility I wrote in order to send the same email to a bunch of companies, who's E-Mail addresses had to be retrieved from a dynamic website.

As to protect my privacy and said companies from spam, the website has not been included in the source and must instead be provided as an argument.

## Usage
Scrape-apply consists of two commands; scrape and apply.
For usage info consult the help commands:

`scrape_apply --help`
```
Usage: scrape_apply <COMMAND>

Commands:
  scrape  scrapes the specified URL
  apply   sends emails to the specified addresses
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

`scrape_apply scrape --help`
```
scrapes the specified URL

Usage: scrape_apply scrape [OPTIONS] <URL>

Arguments:
  <URL>  url to scrape from

Options:
  -o, --output-path <OUTPUT_PATH>  location to save scraped data to [default: ./contact_data.csv]
  -h, --help                       Print help
```

`scrape_apply apply --help`
```
sends emails to the specified addresses

Usage: scrape_apply apply --input-path <INPUT_PATH> --url <URL> --user <USER> --pass <PASS> <MESSAGE>

Arguments:
  <MESSAGE>  .md file with message contents

Options:
  -i, --input-path <INPUT_PATH>  location of input file
      --url <URL>                SMTP server URL
  -u, --user <USER>              SMTP user
  -p, --pass <PASS>              SMTP password
  -h, --help                     Print help
```