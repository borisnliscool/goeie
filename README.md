# Goeie

> *Dutch for 'good one' (good look pronouncing that)*

Super simple redirection service based on DNS. Host this server and set up a CNAME (or A) record pointing to its
location. Additionally, create a `config.toml` file that configures your redirect.

```toml
[website]                       # Any key is fine 'website' is just an example
hosts = ["www.boris.foo"]       # All hosts that redirect to this target
target = "https://boris.foo"    # Where to redirect to
redirect_type = "Temporary"     # Optional; 'Temporary' or 'Permanent', defaults to 'Temporary'
```

#### Planned features:

- caching redirect configuration for hosts to avoid excessive disk I/O
- automatic ssl certs for new sites
- wildcard redirects (e.g., *.boris.foo)