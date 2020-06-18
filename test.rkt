; test utilities
#lang racket

(provide fresh-integer fresh-string fresh-symbol check-contract)

(require rackunit)

(define counter 0)

(define (fresh-integer)
  (set! counter (add1 counter))
  counter)

(define (fresh-string)
  (format "s~a" (fresh-integer)))

(define (fresh-symbol)
  (string->symbol (fresh-string)))

(define-simple-check (check-contract ctrct value)
  (define pred (flat-contract-predicate ctrct))
  (check-pred pred value))