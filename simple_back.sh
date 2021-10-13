#!/bin/bash
DT=full_$(date +%Y_%m_%d)
mkdir /var/backups/$DT
pg_basebackup -D /var/backups/$DT -R -Ft -z -Z 9 