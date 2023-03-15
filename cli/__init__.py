import json
import os.path
import sys
from logging import Logger
from pathlib import Path
from threading import Thread

import colorama
import core
import rich
from core.backend import WeatherForecast

from cli.backend.meteo.meteo_forecast import MeteoForecast
from cli.backend.nws.nws_forecast import NationalWeatherServiceForecast
from cli.backend.openweathermap.openweathermap_forecast import OpenWeatherMapForecast
from cli.backend.theweatherchannel.the_weather_channel_forecast import (
    TheWeatherChannelForecast,
)
from cli.layout.layout import Layout
from cli.local import settings
from cli.local.settings import (
    store_key,
    LAYOUT_FILE,
    AUTO_UPDATE_INTERNET_RESOURCES,
)


def print_out(data: WeatherForecast, print_json: bool, metric: bool, logger: Logger):
    color = colorama.Fore
    if print_json:
        try:
            if isinstance(data.raw_data, list):
                for i in data.raw_data:
                    print(
                        "============================================================"
                    )
                    rich.print_json(json.dumps(i))
            elif isinstance(data.raw_data, str):
                rich.print_json(data.raw_data)
            elif isinstance(data.raw_data, dict):
                rich.print_json(json.dumps(data.raw_data))
            elif isinstance(data.raw_data, core.FormattedData):
                rich.print_json(data.raw_data.raw_data[0])
                rich.print_json(data.raw_data.raw_data[1])
                rich.print_json(data.raw_data.raw_data[2])
            else:
                print(type(data.raw_data))
        except Exception as e:
            print(e)
            print(data.raw_data)
    elif data.status == 0:
        out = Layout(LAYOUT_FILE, logger=logger)
        print(out.to_string(data, metric))
    else:
        print(color.RED + data.raw_data["message"] + color.RESET, end="")


def get_data_from_datasource(datasource, location, true_metric, logger: Logger):
    if not os.path.exists(
        Path(os.path.expanduser("~/.weathercli/weather_codes.json"))
        or Path(os.path.expanduser("~/.weathercli/weather_ascii_images.json"))
    ):
        core.updater.update_web_resources(settings.DEVELOPMENT)
    if AUTO_UPDATE_INTERNET_RESOURCES:
        logger.info("Updating web resources")
        thread = Thread(
            target=core.updater.update_web_resources, args=[settings.DEVELOPMENT]
        )
        thread.start()
    if datasource == "NWS":
        data = NationalWeatherServiceForecast(location, true_metric)
    elif datasource == "THEWEATHERCHANNEL":
        data = TheWeatherChannelForecast(location, true_metric)
    elif datasource == "OPENWEATHERMAP":
        data = OpenWeatherMapForecast(location, true_metric)
    elif datasource == "METEO":
        data = MeteoForecast(location, true_metric)
    else:
        print(colorama.Fore.RED + "Invalid Data Source!")
        logger.critical("Invalid Data Source")
        exit(1)
    logger.info("Data Retrieved")
    if AUTO_UPDATE_INTERNET_RESOURCES:
        thread.join()
    sys.stdout.flush()
    return data
