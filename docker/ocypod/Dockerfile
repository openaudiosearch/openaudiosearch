FROM davechallis/ocypod:latest
RUN apk add --no-cache bash
ADD https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh /opt/bin/
RUN chmod +x /opt/bin/wait-for-it.sh
ENTRYPOINT ["/opt/bin/wait-for-it.sh"]
