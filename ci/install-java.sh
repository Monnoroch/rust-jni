set -e

if [[ "${JDK}" != "openjdk8" ]]; then
    exit 0
fi

# install Java 8
sudo add-apt-repository -y ppa:openjdk-r/ppa
sudo apt-get -qq update
sudo apt-get install -y openjdk-8-jdk --no-install-recommends
