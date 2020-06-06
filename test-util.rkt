#lang racket

(provide fresh-string fresh-symbol)


(define counter 0)

(define (fresh-string)
  (set! counter (add1 counter))
  (format "s~a" counter))

(define (fresh-symbol)
  (string->symbol (fresh-string)))