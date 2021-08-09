FROM rabbitmq:3.7-management
RUN rabbitmq-plugins enable --offline rabbitmq_mqtt rabbitmq_web_mqtt
# Copy certs for SSL, chown then apply config
COPY /etc/letsencrypt/live/tether-io.dev-0001/ /etc/rabbitmq/cert
RUN chown -R rabbitmq:rabbitmq /etc/rabbitmq/cert
COPY ./rabbitmq.conf /etc/rabbitmq