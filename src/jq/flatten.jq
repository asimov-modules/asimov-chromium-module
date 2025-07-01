[
  (.roots.bookmark_bar | recurse(.children[]?) | select(.type == "url")),
  (.roots.other | recurse(.children[]?) | select(.type == "url")) | .
]
