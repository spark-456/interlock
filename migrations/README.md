# Migrations

One SQL file per change, run in filename order by `il-store`:

```
migrations/<YYYYMMDDHHMM>_<module>_<desc>.sql     e.g. 202610271200_fl_flows.sql
```

- Prefix tables with the owning module: `ws_`, `fl_`, `rn_`, `tr_`, `cn_`, `fs_`, `kb_`, `bd_`, `pb_`, `us_` (PRD §10).
- Never edit a merged migration; add a new one.
- `*_kb_*` files belong to K01; everything else to B01 (or the module that adds the table).
