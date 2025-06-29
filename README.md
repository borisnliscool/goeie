# Goeie

> *Dutch for 'good one' (good luck pronouncing that)*

Super simple redirection service. Host this server and set up a CNAME (or A) record pointing to its
location. Additionally, create a `config.toml` file that configures your redirects.

```toml
[[redirect]]
# All hosts that redirect to this target
hosts = ["www.boris.foo"]
# Where to redirect to
target = "https://boris.foo"
# Status to return to the client
# Optional; 'Temporary' or 'Permanent', defaults to 'Temporary'
redirect_type = "Temporary"
# Persist path and query parameters?
# Optional; 'Keep' or 'Remove', defaults to 'Remove'
path = "Keep"

# You can create as many redirects as you want!
[[redirect]]
hosts = ["www.example.com"]
target = "https://example.com"
```

The `config.toml` file is cached in memory for 5 minutes. This means that if you change the file, it might take a few
minutes to propagate. Alternatively, you can restart the server to pick up the new config instantly.

<br/>

#### Planned features:

- automatic ssl certs for new sites
- wildcard redirects (e.g., *.boris.foo)