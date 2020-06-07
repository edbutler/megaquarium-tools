; test utilities
#lang racket

(provide fresh-string fresh-symbol check-contract)

(require rackunit)

(define counter 0)

(define (fresh-string)
  (set! counter (add1 counter))
  (format "s~a" counter))

(define (fresh-symbol)
  (string->symbol (fresh-string)))

(define-simple-check (check-contract ctrct value)
  (define pred (flat-contract-predicate ctrct))
  (check-pred pred value))