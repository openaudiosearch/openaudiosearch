#!/usr/bin/env python3
import pandas as pd
import requests
from bs4 import BeautifulSoup


def request_feed(url: str):
    content_id = url.split("/")[-1]
    feed_url = f"https://cba.fro.at/{content_id}/feed?withoutcomments=1"
    try:
        return requests.get(feed_url)
    except requests.exceptions.RequestException as e:
        print(f"Error fetching {feed_url}")
        print(e)


def parse_feed(response: requests.Response):
    try:
        soup = BeautifulSoup(response.text, "html.parser")
        return soup.find("item")
    except Exception as e:
        print(f"Error parsing XML content of {response.url}")
        print(e)


def fetch_items(urls: list):
    items = []
    for url in urls:
        response = request_feed(url)
        items.append(parse_feed(response))
        print(f"Fetched item from {url}")
    return "\n".join(str(item) for item in items)


def generate_xml(items):
    xml = f"""<?xml version="1.0" encoding="UTF-8"?><rss version="2.0"
		xmlns:content="http://purl.org/rss/1.0/modules/content/"
		xmlns:wfw="http://wellformedweb.org/CommentAPI/"
		xmlns:dc="http://purl.org/dc/elements/1.1/"
		xmlns:atom="http://www.w3.org/2005/Atom"
		xmlns:sy="http://purl.org/rss/1.0/modules/syndication/"
		xmlns:slash="http://purl.org/rss/1.0/modules/slash/"
		
	xmlns:cba="https://cba.fro.at/help#feeds" 
	xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd" 
    xmlns:media="http://search.yahoo.com/mrss/" 
    xmlns:spotify="http://www.spotify.com/ns/rss" 
    xmlns:googleplay="http://www.google.com/schemas/play-podcasts/1.0" >

	<channel>
		<title>OAS Devset RSS Feed</title>
		<link>https://demo.openaudiosearch.org/</link>
		<description>OpenAudioSearch Devset feed to be used for NLP development testing</description>
		<lastBuildDate>Fri, 13 Mar 2009 22:02:21 +0000</lastBuildDate>
		<language>en-US</language>
		<sy:updatePeriod>
		hourly	</sy:updatePeriod>
		<sy:updateFrequency>
		1	</sy:updateFrequency>
		{items}
	</channel>
	</rss>
	"""

    return xml


if __name__ == "__main__":
    """ Generates RSS Feed XML file from spreadsheet containing source URLs """
    source_urls = pd.read_csv("devset/Devset.csv")["URL"].tolist()
    items = fetch_items(source_urls)
    xml = generate_xml(items)
    with open("devset/rss.xml", "w") as f:
        f.write(xml)
