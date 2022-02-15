
import paho.mqtt.client as mqtt 
import time


def on_log(client, userdata, level, buf):
    print("log: ",buf)

def on_connect(client, userdata, flags, rc):
  count = 0
  print("connected with rc", rc)

client = mqtt.Client() #create new instance
# client.on_log = on_log
client.username_pw_set("tether", "sp_ceB0ss!")
# client.on_connect= on_connect

client.connect("tether-io.dev", 1883) #connect to broker
client.loop_start()

while True:
  client.publish("some/topic/address","TEST")#publish
  time.sleep(1)

