# kube-logs-generator

This product was created to use vector.dev with OKD 3.11 but it should be compatible with most (maybe any) versions of Kubernetes.

## Use

You can run this as a container (bkauffman7/kube-logs-generator) or the binary. At the time of this writing, logs is not a configurable path and is `./logs/namespace/pod_name`.

If env variables are not specified lazy static will give an ugly error about binary things.

Here are the env variables needed:

```bash
KUBE_TOKEN=YOUR_TOKEN
KUBE_HOST=https://your_cluser.api.domain.com
CONDITIONS=log_scrape=true,log_no_scrape!=true
```

### Conditions

Conditions are logical AND conditions separated by commas. If you use the `=` operator, the label must exist and match the value. If you use the `!=` the label must not exist or the label must be a different value.
