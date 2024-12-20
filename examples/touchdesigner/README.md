# TouchDesigner Tether example

In order for this to work, you need to install the Pyton Tether Base Agent package into a folder that TouchDesigner is using or is aware of.

For example, a built-in Python installation for TouchDesigner on Mac:
```
cd /Applications/TouchDesigner.app/Contents/Frameworks/Python.framework/Versions/3.11
bin/python3.11 -m pip install tether_agent
```

Even better, use a virtual env for a Python 3.11 version 