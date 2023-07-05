FROM eclipse-mosquitto:2.0.15

WORKDIR /mosquitto/config

# Setup mosquitto config file
RUN echo $'allow_anonymous false' > mosquitto.conf
RUN echo $'password_file /mosquitto/config/mosquitto.passwd' >> mosquitto.conf
RUN echo $'' >> mosquitto.conf
RUN echo $'listener 1883' >> mosquitto.conf
RUN echo $'' >> mosquitto.conf
RUN echo $'listener 9001' >> mosquitto.conf
RUN echo $'protocol websockets' >> mosquitto.conf

# Set credentials to defaults and encode
# RUN echo "${USERNAME}:${PASSWORD}" > mosquitto.passwd
# RUN mosquitto_passwd -U mosquitto.passwd

# create shell script
RUN echo "#!/bin/sh" > entrypoint.sh
# use credentials from arguments
RUN echo "echo \"\$1:\$2\" > /mosquitto/config/mosquitto.passwd" >> entrypoint.sh
# encode passwd file
RUN echo "mosquitto_passwd -U /mosquitto/config/mosquitto.passwd" >> entrypoint.sh
# run cmd from parent image
RUN echo "exec /usr/sbin/mosquitto -c /mosquitto/config/mosquitto.conf" >> entrypoint.sh

# make shell script runnable
RUN chmod +x entrypoint.sh

VOLUME /mosquitto/config

EXPOSE 15675
EXPOSE 1883

# use shell script as entrypoint
ENTRYPOINT /mosquitto/config/entrypoint.sh ${USERNAME:-"guest"} ${PASSWORD:-"guest"}