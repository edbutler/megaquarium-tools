#lang racket

(provide fuzzy-match-species)

(require "animal.rkt")

(define (fuzzy-match-search f search-str lst)
  (define words (string-split search-str " "))
  (filter
    (λ (x)
      (define str (f x))
      (andmap (λ (w) (string-contains?  str w)) words))
    lst))

(define (fuzzy-match-species search-str lst)
  (fuzzy-match-search (compose1 symbol->string species-id) search-str lst))