apiVersion: v1
clusters:
- cluster:
    {{#skip_tls_verify}}
    insecure-skip-tls-verify: true
    {{/skip_tls_verify}}
    server: {{url}}
  name: default_cluster
contexts:
- context:
    cluster: default_cluster
    user: default_user
    namespace: {{namespace}}
  name: default_context
current-context: default_context
kind: Config
preferences: {}
users:
- name: default_user
  user:
    username: {{username}}
    password: {{password}}
