from cli.backend.meteo import Meteo
from cli.layout import Layout
from cli.location import get_location

data = Meteo(get_location(False), False)
l = Layout()
print(l.to_string(data, False))
