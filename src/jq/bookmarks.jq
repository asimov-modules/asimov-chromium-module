{
  "@context": {
    "know": "https://know.dev/",
    "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
    "xsd": "http://www.w3.org/2001/XMLSchema#",
    "items": {
      "@id": "rdfs:member",
      "@type": "know:UserAccount",
      "@container": "@set",
    },
    "created": {
      "@id": "know:created",
      "@type": "xsd:dateTime",
    },
    "title": {
      "@id": "know:title",
      "@language": "en",
    },
    "link": {
      "@id": "know:link",
      "@type": "@id",
    },
  },
  "items": [
    (.roots.bookmark_bar | recurse(.children[]?) | select(.type == "url")),
    (.roots.other | recurse(.children[]?) | select(.type == "url")) | {
      "@id": ("urn:uuid:" + .guid),
      "@type": "know:Bookmark",
      "created": (((.date_added | tonumber) / 1000000) - 11644473600) | todate,
      "title": .name,
      "link": .url,
    }
  ],
}
