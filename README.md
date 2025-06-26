# Goeie

> *Dutch for 'good one' (good look pronouncing that)*

Very simple redirection service based on DNS. Host this server and set up a CNAME (or A) record pointing to its
location. Additionally, create a TXT record specifying the redirection target like this:

```
goeie-redirect-to=https://boris.foo/
```

#### TODO:

- caching redirect configuration for hosts