import argparse
import grpc
from quantify_pb2_grpc import QuantifyDataStub
from quantify_pb2 import Ticker, AddTickerRequest

# TODO: Channel address and port customizatin
SERVICE = "localhost:50051"

parser = argparse.ArgumentParser(
    prog = "quantify-data-cli",
    description=\
"""A minimal CLI client for the quantify-data financial data aggregations service

Can also be used as a library"""
)

parser.add_argument(
    "-a", "--add",
    action='append',
    nargs='*',
    help = "Add ticker"
)

parser.add_argument(
    "-d", "--delete",
    action='append',
    nargs='*',
    help = "Remove ticker" 
)

def add_tickers(qd: QuantifyDataStub, tickers):
    requests = []
    for ticker in tickers:
        grpc_ticker = Ticker(name=ticker)
        grpc_request = AddTickerRequest(ticker=grpc_ticker)
        requests.append(qd.AddTicker.future(grpc_request))
    for future in requests:
        print(future.result())

def remove_tickers(qd: QuantifyDataStub, tickers):
    print("Removing ticker")


# Entry point
def main(args):
    channel = grpc.insecure_channel(SERVICE)
    qd = QuantifyDataStub(channel)

    if args.add:
        add_tickers(qd, args.add[0])
    if args.delete:
        remove_tickers(qd, args.delete[0])

if __name__ == "__main__":
    args = parser.parse_args()
    main(args)
