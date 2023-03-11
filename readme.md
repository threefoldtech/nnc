# `NNC`: Namespace Network Copy

`NNC` is a very simple `network copy` tool that copies traffic from source to destination across network namespaces.

## Operation

`nnc` listens on a certain listen address (ip, port) and accept connections. Once a connection is received the traffic
is redirected to a destination `--target` address. But target is dialed up from a different namespace than the network namespace nnc was started in

The idea behind `nnc` is that it start listening first in the source `namesapce` (this can be the host namespace). To start in a different namespace you can always use the `ip netns exec <ns> nnc ...`

Once `nnc` successfully bind to the listening socket, it switches to the target namespace (provided by the `--namespace` flag). Then any incoming connections from the `public` namespace can be redirected to the `--target` address that is reachable from the private namespace.

## Example

Prepare `priv` namespace

```bash
# create priv namespace
sudo ip netns add priv

# bring lo interface up
sudo ip -n priv l set lo up
```

Let's start a service inside that namespace

```bash
sudo ip netns exec priv python -m http.server --directory /tmp --bind :: 9000
```

This will start an http server that listens on port 9000, and serving files from the `/tmp` directory.

> Feel free to choose another directory to serve

If you now open your browser and tried to connect to `localhost:9000` you will get NOTHING! (ERR_CONNECTION_REFUSED) simply because
the service is listening only INSIDE the `priv` namespace.

Now time to run `nnc`

```bash
sudo nnc -l '[::]:8080' -n /var/run/netns/priv -t 127.0.0.1:9000
```

This basically says, listen on port `8080` (on all interfaces) and once you get a connection, gateway it to `127.0.0.1:9000` inside the `priv` namespace.

> NOTE: the namespaces files locations is platform specific. But it's under /var/run/netns/ on Arch, Ubuntu, and ZOS.

Now try to open `http://locahost:8080` in your browser

If you wish to gateway traffic across 2 namespaces, then simply start `nnc` inside the source namespace. for example

```bash
ip netns exec public nnc -l '[::]:8080' -n /var/run/netns/priv -t 127.0.0.1:9000
```

so it will be listening inside `public` on 8080 and all traffic is redirected to `priv` address `127.0.0.1:9000`
