#!/usr/bin/zsh

yes n | ./pacdef package sync > reply
echo >> reply
diff sync reply
SYNC=$?

./pacdef package unmanaged > reply
diff unmanaged reply
UNMANAGED=$?

exit $SYNC || $UNMANAGED

