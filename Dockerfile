FROM registry.access.redhat.com/ubi8-minimal

ADD vector_mq /

EXPOSE 5555/udp

ENTRYPOINT [ "/vector_mq" ]