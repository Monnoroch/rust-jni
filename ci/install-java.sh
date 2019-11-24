set -e

if [[ "${JDK}" != "java-1.8.0-openjdk-amd64" ]]; then
	exit 0
fi

# install Java 8
sudo add-apt-repository -y ppa:openjdk-r/ppa
sudo apt-get -qq update
sudo apt-get install -y openjdk-8-jdk --no-install-recommends

# change JAVA_HOME to Java 8
export JAVA_HOME=/usr/lib/jvm/java-8-openjdk-amd64

find /usr/lib -name "libjvm.so"
# TODO(https://github.com/rust-lang/cargo/issues/4895): remove this.
export LD_LIBRARY_PATH="$JAVA_HOME/jre/lib/amd64/server"
