from cli.location import reverse_location


# TODO: Port to Rust


class Status:
    OK = 0
    SERVER_ERROR = 1
    INVALID_API_KEY = 2


class WeatherForecast:
    def __init__(
        self,
        status: int,
        region: str,
        country,
        forecast: list,
        forecast_sentence: str,
        raw_data,
    ):
        """
        :param status: 0 is success, 10 is invalid API key, 11 is invalid client request, 20 is server error,
        :param region:
        :param country:
        :param forecast:
        :param forecast_sentence:
        :param raw_data:
        """
        self.status = status
        self.region = region
        self.country = country
        self.forecast = forecast
        self.current_weather = forecast[0]
        self.forecast_sentence = forecast_sentence
        self.raw_data = raw_data

    @staticmethod
    def get_location(loc):
        reversed_location = reverse_location(loc[0], loc[1])
        country = reversed_location["address"]["country"]
        if "city" in reversed_location["address"]:
            region = reversed_location["address"]["city"]
        elif "county" in reversed_location["address"]:
            region = reversed_location["address"]["county"]
        else:
            region = ""
        return region, country
