# Goeie

> *Dutch for 'good one' (good luck pronouncing that)*

Super simple redirection service. Host this server and set up a CNAME (or A) record pointing to its
location. Additionally, create a `config.toml` file that configures your redirects.

```toml
[[redirect]]
hosts = ["www.boris.foo"]    # All hosts that redirect to this target
target = "https://boris.foo" # Where to redirect to
redirect_type = "Temporary"  # Optional; 'Temporary' or 'Permanent', defaults to 'Temporary'

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