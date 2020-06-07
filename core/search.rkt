#lang racket

(provide fuzzy-match-species)

(require
  "animal.rkt"
  "test.rkt")

(define (fuzzy-match-search f search-str lst)
  (define words (string-split search-str " "))
  (filter
    (λ (x)
      (define str (f x))
      (andmap (λ (w) (string-contains?  str w)) words))
    lst))

(define (fuzzy-match-species search-str lst)
  (fuzzy-match-search (compose1 symbol->string species-id) search-str lst))

(module+ test
  (require rackunit)

  (test-case "search works on substrings"
    (define lst '("2_crescent_earthen" "4_pancake_scuppernong" "7_violet_crescent"))
    (check-equal? (fuzzy-match-search identity "pancake" lst) (list (second lst)))
    (check-equal? (fuzzy-match-search identity "olet" lst) (list (third lst)))
    (check-equal? (fuzzy-match-search identity "then" lst) (list (first lst)))
    (check-equal? (fuzzy-match-search identity "cresc" lst) (list (first lst) (third lst)))
    (check-equal? (fuzzy-match-search identity "xyz" lst) empty))

  (test-case "matching species works"
    (define lst
      (map (λ (id) (make-test-species #:id id))
           '(2_crescent_earthen 4_pancake_scuppernong 7_violet_crescent)))
    (check-equal? (fuzzy-match-species "pancake" lst) (list (second lst)))
    (check-equal? (fuzzy-match-species "olet" lst) (list (third lst)))
    (check-equal? (fuzzy-match-species "then" lst) (list (first lst)))
    (check-equal? (fuzzy-match-species "cresc" lst) (list (first lst) (third lst)))
    (check-equal? (fuzzy-match-species "xyz" lst) empty)))
