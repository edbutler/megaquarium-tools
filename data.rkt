#lang racket

(provide
  fuzzy-match-species
  (struct-out game-data)
  read-game-data
  write-tank-yaml
  read-save)

(require
  "core.rkt"
  "data/paths.rkt"
  "data/loader.rkt"
  "data/serialization.rkt")

(struct game-data
  (species
   localization)
  #:transparent) 

(define (species-ref data id)
  (findf
    (λ (a) (equal? id (species-id a)))
    (game-data-species data)))

(define (read-localization data-dir)
  (let* ([map-filename (λ (f) (build-path data-dir (format localization-path-format language f)))]
         [files (map (compose1 read-json-file map-filename) localization-files)])
    (make-localization (cons extra-localization files))))

(define (read-species data-dir)
  (read-species-from-file
    #:animals (build-path data-dir animal-path)
    #:corals (build-path data-dir coral-path)))

(define (read-game-data)
  (define dir (find-data-dir))
  (game-data
    (read-species dir)
    (read-localization dir)))

(define (read-save data name)
  (read-save-from-file (save-file-path) #:species (game-data-species data)))

(module+ test
  (require rackunit racket/contract "test.rkt")
  (test-case "can read data"
    (define data (read-game-data))
    (check-contract (listof species?) (game-data-species data))
    (check-contract localization? (game-data-localization data))))
