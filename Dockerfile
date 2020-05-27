FROM rust:latest
WORKDIR /usr/src/dt-instance
COPY . .

RUN cargo install --path .


FROM debian:buster-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies
COPY --from=builder /usr/local/cargo/bin/myapp /usr/local/bin/myapp
CMD ["dt-instance"]

# WORKDIR /code
# ENV FLASK_APP app.py
# ENV FLASK_RUN_HOST 0.0.0.0
# RUN apk add --no-cache gcc musl-dev linux-headers
# COPY requirements.txt requirements.txt

# RUN pip install -r requirements.txt
# CMD ["flask", "run"]


