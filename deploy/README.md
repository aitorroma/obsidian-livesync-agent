# CouchDB HTTPS deployment (Docker Swarm + Traefik)

This stack deploys CouchDB behind Traefik over HTTPS.

## 1) Create Docker secrets

```bash
printf 'admin' | docker secret create couchdb_user -
printf 'CHANGE_ME_TO_A_STRONG_PASSWORD' | docker secret create couchdb_password -
```

If secrets already exist and you need to rotate them:

```bash
docker secret rm couchdb_user couchdb_password
printf 'admin' | docker secret create couchdb_user -
printf 'NEW_STRONG_PASSWORD' | docker secret create couchdb_password -
```

## 2) Edit host/cert resolver if needed

Open `deploy/couchdb.yaml` and update:

- `traefik.http.routers.couchdb.rule=Host(...)`
- `traefik.http.routers.couchdb.tls.certresolver=...`

## 3) Deploy stack

```bash
docker stack deploy -c deploy/couchdb.yaml couchdb
```

## 4) Check status/logs

```bash
docker service ls | grep couchdb
docker service ps couchdb_couchdb
docker service logs -f couchdb_couchdb
```

## Notes

- Requires existing external network: `proxy`
- Requires Traefik already running in Swarm
- CouchDB data is persisted in volume: `couchdb_data`
