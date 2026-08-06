sed -i '252,255c\
        if let Ok(_) = logger_result { }\
' userland/services/semantic_logger/src/lib.rs
