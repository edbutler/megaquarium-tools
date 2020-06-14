#lang racket

(module+ test
  (provide make-test-tnktyp)
  (require rackunit "../test.rkt"))

(define (temperature? t) (or (equal? t warm-water) (equal? t cold-water)))
(define warm-water 'warm-water)
(define cold-water 'cold-water)

(provide
  (contract-out [temperature? predicate/c]
                [warm-water temperature?]
                [cold-water temperature?]))

; TODO unused
(define salt-water 'salt-water)
(define fresh-water 'fresh-water)
(define (salinity? t) (or (equal? t salt-water) (equal? t fresh-water)))

(provide
  (contract-out [salinity? predicate/c]
                [salt-water salinity?]
                [fresh-water salinity?]))

(struct environment
  (temperature
   quality)
  #:transparent)

(provide
  (contract-out
   [struct environment ((temperature temperature?)
                        (quality (integer-in 0 100)))]))

(module+ test
  (test-case "can create environment"
    (define temp cold-water)
    (define quality 85)
    (define val (environment temp quality))
    (check-eq? (environment-temperature val) temp)
    (check-eq? (environment-quality val) quality)))

(struct tnktyp
  (id
   min-dimensions
   max-dimensions
   volume-per-tile
   rounded?)
  #:transparent)

(define (make-tnktyp
          #:id id
          #:min min-dim
          #:max max-dim
          #:density density
          #:rounded? rounded?)
  (tnktyp id min-dim max-dim density rounded?))

(define (calculate-tank-size kind x-dim y-dim)
  (local-require (only-in racket exact-ceiling))
  (exact-ceiling (* x-dim y-dim (tnktyp-volume-per-tile kind))))

(define dim-pair/c (cons/c exact-positive-integer? exact-positive-integer?))

(provide
  (contract-out
   [struct tnktyp ((id symbol?)
                   (min-dimensions dim-pair/c)
                   (max-dimensions dim-pair/c)
                   (volume-per-tile positive?)
                   (rounded? boolean?))]
   [calculate-tank-size (-> tnktyp?
                            exact-positive-integer?
                            exact-positive-integer?
                            exact-positive-integer?)])
  make-tnktyp)

(module+ test
  (let ()
    (define id 'someid)
    (define min-dim (cons 5 6))
    (define max-dim (cons 7 8))
    (define density 2)
    (define rounded? #t)

    (test-case "can create tnktyp"
      (define val (tnktyp id min-dim max-dim density rounded?))
      (check-eq? (tnktyp-id val) id)
      (check-eq? (tnktyp-min-dimensions val) min-dim)
      (check-eq? (tnktyp-max-dimensions val) max-dim)
      (check-eq? (tnktyp-volume-per-tile val) density)
      (check-eq? (tnktyp-rounded? val) rounded?))

    (test-case "can use make-tnktyp"
      (define val
        (make-tnktyp
          #:id id
          #:min min-dim
          #:max max-dim
          #:density density
          #:rounded? rounded?))
      (define expected (tnktyp id min-dim max-dim density rounded?))
      (check-equal? val expected)))

  (define (make-test-tnktyp
            #:id [id #f]
            #:min [min-dim (cons 2 2)]
            #:max [max-dim (cons 4 4)]
            #:density [density 3]
            #:rounded? [rounded? #f])
    (make-tnktyp
      #:id (or id (fresh-symbol))
      #:min min-dim
      #:max max-dim
      #:density density
      #:rounded? rounded?)))

; A fully specified tank.

(struct tank
  (id
   name
   type
   size
   environment
   lighting)
  #:transparent)

(define (make-tank
          #:id id
          #:name name
          #:kind kind
          #:dimensions [dim #f]
          #:size [size #f]
          #:environment env
          #:lighting light)
  (unless (or dim size) (error "need to define either size or dimensions"))
  (define sz (or size (calculate-tank-size kind (car dim) (cdr dim))))
  (tank id name kind sz env light))

(provide
  (contract-out
   [struct tank ((id integer?)
                 (name string?)
                 (type tnktyp?)
                 (size exact-positive-integer?)
                 (environment environment?)
                 (lighting exact-nonnegative-integer?))])
  make-tank)

(module+ test
  (let ()
    (define id 2534)
    (define name "A name")
    (define kind (make-test-tnktyp #:density 5))
    (define dim (cons 3 6))
    (define size 90)
    (define env (environment warm-water 50))
    (define lighting 15)

    (test-case "can create tank"
      (define val (tank id name kind size env lighting))
      (check-eq? (tank-id val) id)
      (check-eq? (tank-name val) name)
      (check-eq? (tank-type val) kind)
      (check-eq? (tank-size val) size)
      (check-eq? (tank-environment val) env)
      (check-eq? (tank-lighting val) lighting))

    (test-case "can use make-tank with dimensions"
      (define val
        (make-tank
          #:id id
          #:name name
          #:kind kind
          #:dimensions dim
          #:environment env
          #:lighting lighting))
      (define expected (tank id name kind size env lighting))
      (check-equal? val expected))

    (test-case "can use make-tank with size"
      (define val
        (make-tank
          #:id id
          #:name name
          #:kind kind
          #:size size
          #:environment env
          #:lighting lighting))
      (define expected (tank id name kind size env lighting))
      (check-equal? val expected))))
